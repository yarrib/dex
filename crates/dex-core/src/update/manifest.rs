//! Project update state: `.dex/manifest.toml`, `.dex/history.toml`, and the
//! rendered baseline cache under `.dex/cache/baseline/`.

use serde::Deserialize;
use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};

use crate::config::chrono_date_today;
use crate::error::{ConfigError, DexError, UpdateError};
use crate::scaffold::{RenderedTree, render_tree, write_tree};
use crate::template::variables::{VariableSpec, VariableType};
use crate::template::{HooksSpec, Template};

/// Project-local state directory name.
pub const STATE_DIR: &str = ".dex";
const MANIFEST_FILE: &str = "manifest.toml";
const HISTORY_FILE: &str = "history.toml";
const BASELINE_SUBDIR: &str = "cache/baseline";
/// Current `.dex/manifest.toml` schema version.
pub const SCHEMA_VERSION: u32 = 1;

/// Where the project's template came from, as recorded in the manifest.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SourceKind {
    /// Built into the dex binary; `ref` is the template's version.
    Embedded,
    /// A local template directory; `ref` is the template's version.
    Directory,
    /// A remote git repository; `ref` is the resolved commit SHA.
    Remote,
}

impl SourceKind {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            SourceKind::Embedded => "embedded",
            SourceKind::Directory => "directory",
            SourceKind::Remote => "remote",
        }
    }
}

/// The `[template]` section of `.dex/manifest.toml`.
#[derive(Debug, Clone, Deserialize)]
pub struct TemplateState {
    pub name: String,
    pub source: SourceKind,
    /// Remote: repository URL. Directory: absolute path. Embedded: absent.
    #[serde(default)]
    pub location: Option<String>,
    /// Pinned ref used at last generation/update: a commit SHA for remote
    /// sources, the template version for embedded/directory sources.
    #[serde(rename = "ref")]
    pub git_ref: String,
    /// `[template].version` at that ref (informational).
    #[serde(default)]
    pub version: Option<String>,
    /// dex binary version that generated the project.
    #[serde(default)]
    pub dex_version: Option<String>,
}

/// The `[hooks]` section of `.dex/manifest.toml`.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct UpdateHooks {
    #[serde(default)]
    pub pre_update: Option<String>,
    #[serde(default)]
    pub post_update: Option<String>,
}

impl From<&HooksSpec> for UpdateHooks {
    fn from(spec: &HooksSpec) -> Self {
        UpdateHooks {
            pre_update: spec.pre_update.clone(),
            post_update: spec.post_update.clone(),
        }
    }
}

impl UpdateHooks {
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.pre_update.is_none() && self.post_update.is_none()
    }
}

/// Parsed `.dex/manifest.toml` — the update-critical project state.
#[derive(Debug, Deserialize)]
pub struct StateManifest {
    #[serde(default = "schema_version_default")]
    pub schema_version: u32,
    pub template: TemplateState,
    /// Typed answers from the last generation/update, keyed by variable name.
    #[serde(default)]
    pub answers: BTreeMap<String, toml::Value>,
    #[serde(default)]
    pub hooks: UpdateHooks,
}

fn schema_version_default() -> u32 {
    SCHEMA_VERSION
}

/// One entry in `.dex/history.toml`.
#[derive(Debug, Clone, Deserialize)]
pub struct HistoryEntry {
    pub date: String,
    pub from_ref: String,
    pub to_ref: String,
    #[serde(default)]
    pub dex_version: Option<String>,
    #[serde(default)]
    pub files_updated: usize,
    #[serde(default)]
    pub files_conflicted: usize,
}

#[derive(Debug, Default, Deserialize)]
struct HistoryFile {
    #[serde(default)]
    update: Vec<HistoryEntry>,
}

// ---------------------------------------------------------------------------
// Paths
// ---------------------------------------------------------------------------

#[must_use]
pub fn state_dir(project_dir: &Path) -> PathBuf {
    project_dir.join(STATE_DIR)
}

#[must_use]
pub fn manifest_path(project_dir: &Path) -> PathBuf {
    state_dir(project_dir).join(MANIFEST_FILE)
}

#[must_use]
pub fn history_path(project_dir: &Path) -> PathBuf {
    state_dir(project_dir).join(HISTORY_FILE)
}

#[must_use]
pub fn baseline_dir(project_dir: &Path) -> PathBuf {
    state_dir(project_dir).join(BASELINE_SUBDIR)
}

// ---------------------------------------------------------------------------
// Manifest read/write
// ---------------------------------------------------------------------------

/// Load `.dex/manifest.toml` from a project directory.
pub fn load_state_manifest(project_dir: &Path) -> Result<StateManifest, DexError> {
    let path = manifest_path(project_dir);
    let content = std::fs::read_to_string(&path).map_err(|source| {
        if source.kind() == std::io::ErrorKind::NotFound {
            DexError::Update(UpdateError::NoManifest(path.clone()))
        } else {
            DexError::Io {
                path: path.clone(),
                source,
            }
        }
    })?;

    let manifest: StateManifest = toml::from_str(&content).map_err(ConfigError::Parse)?;
    Ok(manifest)
}

/// Write `.dex/manifest.toml`, creating `.dex/` if needed.
///
/// Deterministic output: answers are written in sorted key order. Comments in
/// an existing manifest are not preserved (same trade-off as `record_trait`).
pub fn save_state_manifest(project_dir: &Path, manifest: &StateManifest) -> Result<(), DexError> {
    let dir = state_dir(project_dir);
    std::fs::create_dir_all(&dir).map_err(|source| DexError::Io {
        path: dir.clone(),
        source,
    })?;

    let mut lines = vec![
        "# dex project state — written by `dex init`, updated by `dex update`.".to_string(),
        "# Commit this file; `.dex/cache/` is ignored via `.dex/.gitignore`.".to_string(),
        format!("schema_version = {}", manifest.schema_version),
        String::new(),
        "[template]".to_string(),
        format!("name = {:?}", manifest.template.name),
        format!("source = {:?}", manifest.template.source.as_str()),
    ];
    if let Some(location) = &manifest.template.location {
        lines.push(format!("location = {location:?}"));
    }
    lines.push(format!("ref = {:?}", manifest.template.git_ref));
    if let Some(version) = &manifest.template.version {
        lines.push(format!("version = {version:?}"));
    }
    if let Some(dex_version) = &manifest.template.dex_version {
        lines.push(format!("dex_version = {dex_version:?}"));
    }

    lines.push(String::new());
    lines.push("[answers]".to_string());
    for (k, v) in &manifest.answers {
        lines.push(format_toml_pair(k, v));
    }

    if !manifest.hooks.is_empty() {
        lines.push(String::new());
        lines.push("[hooks]".to_string());
        if let Some(pre) = &manifest.hooks.pre_update {
            lines.push(format!("pre_update = {pre:?}"));
        }
        if let Some(post) = &manifest.hooks.post_update {
            lines.push(format!("post_update = {post:?}"));
        }
    }

    lines.push(String::new());
    let path = manifest_path(project_dir);
    std::fs::write(&path, lines.join("\n")).map_err(|source| DexError::Io { path, source })?;
    Ok(())
}

fn format_toml_pair(key: &str, value: &toml::Value) -> String {
    match value {
        toml::Value::Boolean(b) => format!("{key} = {b}"),
        toml::Value::String(s) => format!("{key} = {s:?}"),
        toml::Value::Integer(i) => format!("{key} = {i}"),
        toml::Value::Float(f) => format!("{key} = {f}"),
        other => format!("{key} = {:?}", other.to_string()),
    }
}

// ---------------------------------------------------------------------------
// History
// ---------------------------------------------------------------------------

const HISTORY_HEADER: &str = "# dex update history — append-only log of template updates.\n";

/// Append an entry to `.dex/history.toml`, creating the file if needed.
pub fn append_history(project_dir: &Path, entry: &HistoryEntry) -> Result<(), DexError> {
    let path = history_path(project_dir);
    let dir = state_dir(project_dir);
    std::fs::create_dir_all(&dir).map_err(|source| DexError::Io {
        path: dir.clone(),
        source,
    })?;

    let mut content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => HISTORY_HEADER.to_string(),
        Err(source) => {
            return Err(DexError::Io {
                path: path.clone(),
                source,
            });
        }
    };

    if !content.is_empty() && !content.ends_with('\n') {
        content.push('\n');
    }
    content.push_str("\n[[update]]\n");
    content.push_str(&format!("date = {:?}\n", entry.date));
    content.push_str(&format!("from_ref = {:?}\n", entry.from_ref));
    content.push_str(&format!("to_ref = {:?}\n", entry.to_ref));
    if let Some(v) = &entry.dex_version {
        content.push_str(&format!("dex_version = {v:?}\n"));
    }
    content.push_str(&format!("files_updated = {}\n", entry.files_updated));
    content.push_str(&format!("files_conflicted = {}\n", entry.files_conflicted));

    std::fs::write(&path, content).map_err(|source| DexError::Io { path, source })?;
    Ok(())
}

/// Read all entries from `.dex/history.toml` (empty if the file is missing).
pub fn load_history(project_dir: &Path) -> Result<Vec<HistoryEntry>, DexError> {
    let path = history_path(project_dir);
    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(source) => return Err(DexError::Io { path, source }),
    };
    let parsed: HistoryFile = toml::from_str(&content).map_err(ConfigError::Parse)?;
    Ok(parsed.update)
}

/// Convenience for building a history entry dated today.
#[must_use]
pub fn history_entry_today(
    from_ref: &str,
    to_ref: &str,
    dex_version: Option<&str>,
    files_updated: usize,
    files_conflicted: usize,
) -> HistoryEntry {
    HistoryEntry {
        date: chrono_date_today(),
        from_ref: from_ref.to_string(),
        to_ref: to_ref.to_string(),
        dex_version: dex_version.map(str::to_string),
        files_updated,
        files_conflicted,
    }
}

// ---------------------------------------------------------------------------
// Baseline cache
// ---------------------------------------------------------------------------

/// Replace `.dex/cache/baseline/` with the given rendered tree.
///
/// The cache is always rewritten whole so it can never mix files from two
/// template versions.
pub fn write_baseline_cache(project_dir: &Path, tree: &RenderedTree) -> Result<(), DexError> {
    let dir = baseline_dir(project_dir);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).map_err(|source| DexError::Io {
            path: dir.clone(),
            source,
        })?;
    }
    std::fs::create_dir_all(&dir).map_err(|source| DexError::Io {
        path: dir.clone(),
        source,
    })?;
    write_tree(tree, &dir)?;
    Ok(())
}

/// Read the baseline cache back, if present.
pub fn load_baseline_cache(project_dir: &Path) -> Result<Option<RenderedTree>, DexError> {
    let dir = baseline_dir(project_dir);
    if !dir.is_dir() {
        return Ok(None);
    }
    Ok(Some(RenderedTree::from_dir(&dir)?))
}

// ---------------------------------------------------------------------------
// Init-time state write
// ---------------------------------------------------------------------------

/// Write the full `.dex/` state for a freshly generated project: manifest,
/// `.gitignore` (ignoring `cache/`), an empty history file, and — when a
/// rendered baseline is provided — the offline baseline cache.
pub fn write_project_state(
    project_dir: &Path,
    manifest: &StateManifest,
    baseline: Option<&RenderedTree>,
) -> Result<(), DexError> {
    save_state_manifest(project_dir, manifest)?;

    let gitignore = state_dir(project_dir).join(".gitignore");
    if !gitignore.exists() {
        std::fs::write(&gitignore, "cache/\n").map_err(|source| DexError::Io {
            path: gitignore.clone(),
            source,
        })?;
    }

    let history = history_path(project_dir);
    if !history.exists() {
        std::fs::write(&history, HISTORY_HEADER).map_err(|source| DexError::Io {
            path: history.clone(),
            source,
        })?;
    }

    if let Some(tree) = baseline {
        write_baseline_cache(project_dir, tree)?;
    }

    Ok(())
}

/// Build the init-time state manifest for a freshly scaffolded project.
#[must_use]
pub fn build_state_manifest(
    template: &Template,
    source: SourceKind,
    location: Option<String>,
    git_ref: String,
    dex_version: &str,
    variables: &HashMap<String, minijinja::Value>,
) -> StateManifest {
    StateManifest {
        schema_version: SCHEMA_VERSION,
        template: TemplateState {
            name: template.meta.name.clone(),
            source,
            location,
            git_ref,
            version: Some(template.meta.version.clone()),
            dex_version: Some(dex_version.to_string()),
        },
        answers: typed_answers(&template.variables, variables),
        hooks: template
            .hooks
            .as_ref()
            .map(UpdateHooks::from)
            .unwrap_or_default(),
    }
}

/// Record the full `.dex/` state (manifest + history + baseline cache) for a
/// project just scaffolded from `template` with `variables`. One call for
/// every scaffold entry point (`dex init`, the MCP `scaffold_project` tool).
pub fn record_project_state(
    project_dir: &Path,
    template: &Template,
    source: SourceKind,
    location: Option<String>,
    git_ref: String,
    dex_version: &str,
    variables: &HashMap<String, minijinja::Value>,
) -> Result<(), DexError> {
    let manifest =
        build_state_manifest(template, source, location, git_ref, dex_version, variables);
    let baseline = render_tree(template, variables)?;
    write_project_state(project_dir, &manifest, Some(&baseline))
}

/// Convert resolved template variables into typed TOML answers, in the same
/// bool-or-string convention used by `save_answers` (`config.rs`): bools keep
/// their type so replay gates `[[files]]` conditions correctly; everything
/// else is stored as a string.
#[must_use]
pub fn typed_answers(
    specs: &[VariableSpec],
    values: &HashMap<String, minijinja::Value>,
) -> BTreeMap<String, toml::Value> {
    let mut answers = BTreeMap::new();
    for spec in specs {
        if let Some(v) = values.get(&spec.name) {
            let toml_val = match spec.var_type {
                VariableType::Bool => toml::Value::Boolean(v.is_true()),
                _ => toml::Value::String(
                    v.as_str()
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| v.to_string()),
                ),
            };
            answers.insert(spec.name.clone(), toml_val);
        }
    }
    answers
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_manifest() -> StateManifest {
        let mut answers = BTreeMap::new();
        answers.insert(
            "project_name".to_string(),
            toml::Value::String("my_proj".to_string()),
        );
        answers.insert("include_notebook".to_string(), toml::Value::Boolean(true));

        StateManifest {
            schema_version: 1,
            template: TemplateState {
                name: "dabs-package".to_string(),
                source: SourceKind::Remote,
                location: Some("https://example.com/templates.git".to_string()),
                git_ref: "9f3ab12deadbeef".to_string(),
                version: Some("0.3.0".to_string()),
                dex_version: Some("0.6.0".to_string()),
            },
            answers,
            hooks: UpdateHooks {
                pre_update: None,
                post_update: Some("uv sync".to_string()),
            },
        }
    }

    #[test]
    fn manifest_round_trip_preserves_typed_answers() {
        let dir = tempfile::tempdir().unwrap();
        let manifest = sample_manifest();

        save_state_manifest(dir.path(), &manifest).unwrap();
        let loaded = load_state_manifest(dir.path()).unwrap();

        assert_eq!(loaded.schema_version, 1);
        assert_eq!(loaded.template.name, "dabs-package");
        assert_eq!(loaded.template.source, SourceKind::Remote);
        assert_eq!(
            loaded.template.location.as_deref(),
            Some("https://example.com/templates.git")
        );
        assert_eq!(loaded.template.git_ref, "9f3ab12deadbeef");
        assert_eq!(loaded.template.version.as_deref(), Some("0.3.0"));
        assert_eq!(
            loaded.answers.get("project_name"),
            Some(&toml::Value::String("my_proj".to_string()))
        );
        // Bool stays a bool — this is what keeps `[[files]]` conditions
        // gating correctly on replay.
        assert_eq!(
            loaded.answers.get("include_notebook"),
            Some(&toml::Value::Boolean(true))
        );
        assert_eq!(loaded.hooks.post_update.as_deref(), Some("uv sync"));
        assert!(loaded.hooks.pre_update.is_none());
    }

    #[test]
    fn load_missing_manifest_is_no_manifest_error() {
        let dir = tempfile::tempdir().unwrap();
        let result = load_state_manifest(dir.path());
        assert!(
            matches!(result, Err(DexError::Update(UpdateError::NoManifest(_)))),
            "expected UpdateError::NoManifest"
        );
    }

    #[test]
    fn history_append_is_append_only() {
        let dir = tempfile::tempdir().unwrap();

        let first = history_entry_today("aaa", "bbb", Some("0.6.0"), 4, 1);
        append_history(dir.path(), &first).unwrap();
        let second = history_entry_today("bbb", "ccc", Some("0.6.1"), 2, 0);
        append_history(dir.path(), &second).unwrap();

        let entries = load_history(dir.path()).unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].from_ref, "aaa");
        assert_eq!(entries[0].to_ref, "bbb");
        assert_eq!(entries[0].files_updated, 4);
        assert_eq!(entries[0].files_conflicted, 1);
        assert_eq!(entries[1].from_ref, "bbb");
        assert_eq!(entries[1].to_ref, "ccc");
    }

    #[test]
    fn write_project_state_creates_all_files() {
        let dir = tempfile::tempdir().unwrap();
        let manifest = sample_manifest();

        let mut files = BTreeMap::new();
        files.insert(
            std::path::PathBuf::from("README.md"),
            "# hello\n".to_string(),
        );
        let tree = RenderedTree { files };

        write_project_state(dir.path(), &manifest, Some(&tree)).unwrap();

        assert!(manifest_path(dir.path()).is_file());
        assert!(history_path(dir.path()).is_file());
        assert_eq!(
            std::fs::read_to_string(state_dir(dir.path()).join(".gitignore")).unwrap(),
            "cache/\n"
        );
        assert_eq!(
            std::fs::read_to_string(baseline_dir(dir.path()).join("README.md")).unwrap(),
            "# hello\n"
        );

        let cached = load_baseline_cache(dir.path()).unwrap().unwrap();
        assert_eq!(cached, tree);
    }

    #[test]
    fn baseline_cache_is_replaced_whole() {
        let dir = tempfile::tempdir().unwrap();

        let mut files = BTreeMap::new();
        files.insert(std::path::PathBuf::from("old.txt"), "old\n".to_string());
        write_baseline_cache(dir.path(), &RenderedTree { files }).unwrap();

        let mut files = BTreeMap::new();
        files.insert(std::path::PathBuf::from("new.txt"), "new\n".to_string());
        write_baseline_cache(dir.path(), &RenderedTree { files }).unwrap();

        let cached = load_baseline_cache(dir.path()).unwrap().unwrap();
        assert!(cached.files.contains_key(std::path::Path::new("new.txt")));
        assert!(
            !cached.files.contains_key(std::path::Path::new("old.txt")),
            "stale baseline file survived a cache rewrite"
        );
    }

    #[test]
    fn typed_answers_preserves_bools_and_strings() {
        use crate::template::variables::VariableSpec;

        let specs = vec![
            VariableSpec {
                name: "project_name".to_string(),
                prompt: "Name".to_string(),
                var_type: VariableType::String,
                default: None,
                required: true,
                choices: None,
                validate: None,
                order: None,
                when: None,
            },
            VariableSpec {
                name: "use_ci".to_string(),
                prompt: "CI?".to_string(),
                var_type: VariableType::Bool,
                default: None,
                required: false,
                choices: None,
                validate: None,
                order: None,
                when: None,
            },
        ];

        let mut values = HashMap::new();
        values.insert("project_name".to_string(), minijinja::Value::from("proj"));
        values.insert("use_ci".to_string(), minijinja::Value::from(false));
        values.insert("unknown".to_string(), minijinja::Value::from("ignored"));

        let answers = typed_answers(&specs, &values);
        assert_eq!(
            answers.get("project_name"),
            Some(&toml::Value::String("proj".to_string()))
        );
        assert_eq!(answers.get("use_ci"), Some(&toml::Value::Boolean(false)));
        assert!(!answers.contains_key("unknown"));
    }
}
