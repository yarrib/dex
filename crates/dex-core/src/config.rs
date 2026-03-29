//! Project and user configuration (`dex.toml`, `~/.config/dex/config.toml`).

use serde::Deserialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::error::{ConfigError, DexError};

// ---------------------------------------------------------------------------
// Project config (dex.toml)
// ---------------------------------------------------------------------------

/// Top-level project configuration from `dex.toml`.
#[derive(Debug, Deserialize)]
pub struct ProjectConfig {
    pub project: ProjectMeta,

    #[serde(default)]
    pub tasks: HashMap<String, TaskSpec>,

    #[serde(default)]
    pub profiles: HashMap<String, ProfileSpec>,

    #[serde(default)]
    pub passthrough: HashMap<String, PassthroughSpec>,
}

/// Project metadata from `[project]` section.
#[derive(Debug, Deserialize)]
pub struct ProjectMeta {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub template: Option<String>,
}

/// Task definition from `[tasks.*]` section.
#[derive(Debug, Deserialize)]
pub struct TaskSpec {
    pub command: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub depends_on: Vec<String>,
}

/// Environment profile from `[profiles.*]` section.
#[derive(Debug, Deserialize)]
pub struct ProfileSpec {
    #[serde(default)]
    pub workspace_url: Option<String>,
    #[serde(default)]
    pub cluster_id: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, toml::Value>,
}

/// Pass-through command spec from `[passthrough.*]` section.
#[derive(Debug, Clone, Deserialize)]
pub struct PassthroughSpec {
    pub command: String,
    #[serde(default)]
    pub description: Option<String>,
}

/// Load and parse a `dex.toml` project config from a file path.
pub fn load_project_config(path: &Path) -> Result<ProjectConfig, DexError> {
    let content = std::fs::read_to_string(path).map_err(|source| {
        if source.kind() == std::io::ErrorKind::NotFound {
            DexError::Config(ConfigError::NotFound(path.to_path_buf()))
        } else {
            DexError::Io {
                path: path.to_path_buf(),
                source,
            }
        }
    })?;

    let config: ProjectConfig = toml::from_str(&content).map_err(ConfigError::Parse)?;
    Ok(config)
}

// ---------------------------------------------------------------------------
// User / merged config
// ---------------------------------------------------------------------------

/// A git repository containing dex templates.
#[derive(Debug, Clone)]
pub struct RemoteSource {
    pub name: String,
    pub url: String,
    pub git_ref: Option<String>,
}

/// Resolved dex configuration (user + project merged).
#[derive(Debug, Clone, Default)]
pub struct DexConfig {
    pub templates_dir: Option<PathBuf>,
    pub remotes: Vec<RemoteSource>,
}

/// Well-known paths.
pub fn user_config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("~/.config"))
        .join("dex")
        .join("config.toml")
}

pub fn standards_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("~/.config"))
        .join("dex")
        .join("standards.toml")
}

pub fn presets_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("~/.config"))
        .join("dex")
        .join("presets.toml")
}

pub fn remote_cache_dir() -> PathBuf {
    dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("~/.cache"))
        .join("dex")
        .join("templates")
}

/// Load variable pre-fills from a standards TOML file.
///
/// Returns a flat map of variable name → string value.
pub fn load_standards(path: Option<&Path>) -> Result<HashMap<String, String>, DexError> {
    let target = match path {
        Some(p) => p.to_path_buf(),
        None => standards_path(),
    };

    if !target.exists() {
        return Ok(HashMap::new());
    }

    let content = std::fs::read_to_string(&target).map_err(|source| DexError::Io {
        path: target.clone(),
        source,
    })?;

    let table: HashMap<String, toml::Value> =
        toml::from_str(&content).map_err(ConfigError::Parse)?;

    Ok(table
        .into_iter()
        .map(|(k, v)| (k, toml_value_to_string(&v)))
        .collect())
}

/// Load variable pre-fills for a named preset profile.
///
/// Presets group related variable values under named profiles so you can
/// pre-fill different sets of variables depending on the project type:
///
/// ```toml
/// # ~/.config/dex/presets.toml
///
/// [profiles.ml-project]
/// workspace_url  = "https://ml.cloud.databricks.com"
/// cluster_id     = "0123-456789-ml"
/// python_version = "3.12"
///
/// [profiles.etl]
/// workspace_url  = "https://etl.cloud.databricks.com"
/// python_version = "3.11"
/// ```
///
/// Select a profile with `dex init --preset ml-project`.
///
/// Returns an error if the file does not exist or the named profile is missing.
pub fn load_preset(
    path: Option<&Path>,
    profile: &str,
) -> Result<HashMap<String, String>, DexError> {
    let target = match path {
        Some(p) => p.to_path_buf(),
        None => presets_path(),
    };

    if !target.exists() {
        return Err(DexError::Config(ConfigError::NotFound(target)));
    }

    let content = std::fs::read_to_string(&target).map_err(|source| DexError::Io {
        path: target.clone(),
        source,
    })?;

    let table: toml::Table = toml::from_str(&content).map_err(ConfigError::Parse)?;

    let profiles = table
        .get("profiles")
        .and_then(|v| v.as_table())
        .ok_or_else(|| {
            DexError::Config(ConfigError::Invalid(
                "presets file has no [profiles] section".to_string(),
            ))
        })?;

    let profile_table = profiles
        .get(profile)
        .and_then(|v| v.as_table())
        .ok_or_else(|| {
            let available: Vec<&str> = profiles.keys().map(String::as_str).collect();
            DexError::Config(ConfigError::Invalid(format!(
                "preset '{}' not found. Available: {}",
                profile,
                if available.is_empty() {
                    "(none)".to_string()
                } else {
                    available.join(", ")
                }
            )))
        })?;

    Ok(profile_table
        .iter()
        .map(|(k, v)| (k.clone(), toml_value_to_string(v)))
        .collect())
}

/// Load and merge user config (`~/.config/dex/config.toml`) and project
/// config (`./dex.toml`). Project config takes precedence; remotes are additive.
pub fn load_dex_config() -> DexConfig {
    let user = parse_config_file(&user_config_path());
    let project = parse_config_file(&PathBuf::from("dex.toml"));
    merge_configs(user, project)
}

fn parse_config_file(path: &Path) -> DexConfig {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return DexConfig::default(),
    };

    let data: toml::Table = match toml::from_str(&content) {
        Ok(t) => t,
        Err(_) => return DexConfig::default(),
    };

    let templates = data
        .get("templates")
        .and_then(|v| v.as_table())
        .cloned()
        .unwrap_or_default();

    let templates_dir = templates.get("dir").and_then(|v| v.as_str()).map(|s| {
        let expanded = shellexpand::tilde(s);
        PathBuf::from(expanded.as_ref())
    });

    let remotes = templates
        .get("remotes")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|item| {
                    let t = item.as_table()?;
                    let name = t.get("name")?.as_str()?.to_string();
                    let url = t.get("url")?.as_str()?.to_string();
                    let git_ref = t.get("ref").and_then(|v| v.as_str()).map(String::from);
                    Some(RemoteSource { name, url, git_ref })
                })
                .collect()
        })
        .unwrap_or_default();

    DexConfig {
        templates_dir,
        remotes,
    }
}

fn merge_configs(user: DexConfig, project: DexConfig) -> DexConfig {
    let project_names: std::collections::HashSet<_> =
        project.remotes.iter().map(|r| r.name.clone()).collect();
    let mut merged_remotes = project.remotes;
    merged_remotes.extend(
        user.remotes
            .into_iter()
            .filter(|r| !project_names.contains(&r.name)),
    );

    DexConfig {
        templates_dir: project.templates_dir.or(user.templates_dir),
        remotes: merged_remotes,
    }
}

/// Clone or update a remote template repository, returning the local cache path.
pub fn resolve_remote(remote: &RemoteSource, update: bool) -> Result<PathBuf, DexError> {
    let dest = remote_cache_dir().join(&remote.name);

    if dest.exists() {
        if update {
            git_pull(&dest, remote.git_ref.as_deref());
        }
    } else {
        std::fs::create_dir_all(dest.parent().unwrap_or(&dest)).map_err(|source| DexError::Io {
            path: dest.clone(),
            source,
        })?;
        git_clone(&remote.url, &dest, remote.git_ref.as_deref())?;
    }

    Ok(dest)
}

fn git_clone(url: &str, dest: &Path, git_ref: Option<&str>) -> Result<(), DexError> {
    let mut cmd = std::process::Command::new("git");
    cmd.args(["clone", "--depth", "1"]);
    if let Some(r) = git_ref {
        cmd.args(["--branch", r]);
    }
    cmd.arg(url).arg(dest);

    let output = cmd.output().map_err(|source| DexError::Io {
        path: dest.to_path_buf(),
        source,
    })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(DexError::Config(ConfigError::Invalid(format!(
            "failed to clone template repo '{url}': {stderr}"
        ))));
    }

    Ok(())
}

fn git_pull(dest: &Path, git_ref: Option<&str>) {
    // Non-fatal: stale cache is better than no cache.
    let dest_str = dest.to_string_lossy().to_string();

    if let Some(r) = git_ref {
        let _ = std::process::Command::new("git")
            .args(["-C", &dest_str, "fetch", "--depth", "1", "origin", r])
            .output();
    } else {
        let _ = std::process::Command::new("git")
            .args(["-C", &dest_str, "pull", "--ff-only"])
            .output();
    }
}

fn toml_value_to_string(v: &toml::Value) -> String {
    match v {
        toml::Value::String(s) => s.clone(),
        toml::Value::Boolean(b) => b.to_string(),
        toml::Value::Integer(i) => i.to_string(),
        toml::Value::Float(f) => f.to_string(),
        other => other.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_minimal_config() {
        let toml_str = r#"
            [project]
            name = "my-project"
        "#;
        let config: ProjectConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.project.name, "my-project");
        assert!(config.tasks.is_empty());
        assert!(config.profiles.is_empty());
    }

    #[test]
    fn parse_full_config() {
        let toml_str = r#"
            [project]
            name = "ml-pipeline"
            description = "Revenue forecasting"
            template = "ml-pipeline"

            [tasks.test]
            command = "pytest tests/"
            description = "Run tests"

            [tasks.build]
            command = "python -m build"
            depends_on = ["test"]

            [profiles.dev]
            workspace_url = "https://dev.cloud.databricks.com"
            cluster_id = "0123-456789-abcdef"

            [passthrough.db]
            command = "databricks"
            description = "Databricks CLI"
        "#;
        let config: ProjectConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.project.name, "ml-pipeline");
        assert_eq!(config.tasks.len(), 2);
        assert_eq!(config.tasks["build"].depends_on, vec!["test"]);
        assert_eq!(
            config.profiles["dev"].cluster_id.as_deref(),
            Some("0123-456789-abcdef")
        );
        assert_eq!(config.passthrough["db"].command, "databricks");
    }

    #[test]
    fn merge_configs_project_overrides_user() {
        let user = DexConfig {
            templates_dir: Some(PathBuf::from("/user/templates")),
            remotes: vec![RemoteSource {
                name: "shared".into(),
                url: "https://example.com/shared".into(),
                git_ref: None,
            }],
        };
        let project = DexConfig {
            templates_dir: Some(PathBuf::from("/project/templates")),
            remotes: vec![],
        };
        let merged = merge_configs(user, project);
        assert_eq!(
            merged.templates_dir,
            Some(PathBuf::from("/project/templates"))
        );
        assert_eq!(merged.remotes.len(), 1);
        assert_eq!(merged.remotes[0].name, "shared");
    }

    #[test]
    fn load_preset_happy_path() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("presets.toml");
        std::fs::write(
            &path,
            r#"
[profiles.ml-project]
workspace_url = "https://ml.cloud.databricks.com"
cluster_id = "0123-ml"
python_version = "3.12"

[profiles.etl]
workspace_url = "https://etl.cloud.databricks.com"
"#,
        )
        .unwrap();

        let preset = load_preset(Some(&path), "ml-project").unwrap();
        assert_eq!(
            preset.get("workspace_url").unwrap(),
            "https://ml.cloud.databricks.com"
        );
        assert_eq!(preset.get("cluster_id").unwrap(), "0123-ml");
        assert_eq!(preset.get("python_version").unwrap(), "3.12");

        // Other profile is unaffected
        let etl = load_preset(Some(&path), "etl").unwrap();
        assert_eq!(
            etl.get("workspace_url").unwrap(),
            "https://etl.cloud.databricks.com"
        );
        assert!(!etl.contains_key("cluster_id"));
    }

    #[test]
    fn load_preset_unknown_profile_returns_error() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("presets.toml");
        std::fs::write(
            &path,
            "[profiles.ml-project]\nworkspace_url = \"https://example.com\"\n",
        )
        .unwrap();
        let result = load_preset(Some(&path), "does-not-exist");
        assert!(
            matches!(result, Err(DexError::Config(ConfigError::Invalid(_)))),
            "expected ConfigError::Invalid for unknown profile"
        );
    }

    #[test]
    fn load_preset_missing_file_returns_error() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("nonexistent.toml");
        let result = load_preset(Some(&path), "ml-project");
        assert!(
            matches!(result, Err(DexError::Config(ConfigError::NotFound(_)))),
            "expected ConfigError::NotFound for missing file"
        );
    }

    #[test]
    fn load_project_config_not_found() {
        let result = load_project_config(Path::new("/nonexistent/dex.toml"));
        assert!(
            matches!(result, Err(DexError::Config(ConfigError::NotFound(_)))),
            "expected ConfigError::NotFound"
        );
    }

    #[test]
    fn load_project_config_invalid_toml() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("dex.toml");
        std::fs::write(&path, "this is not valid toml ][[[").unwrap();
        let result = load_project_config(&path);
        assert!(
            matches!(result, Err(DexError::Config(ConfigError::Parse(_)))),
            "expected ConfigError::Parse"
        );
    }

    #[test]
    fn load_standards_missing_path_returns_empty() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("nonexistent.toml");
        let standards = load_standards(Some(&path)).unwrap();
        assert!(standards.is_empty());
    }

    #[test]
    fn load_standards_from_string() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("standards.toml");
        std::fs::write(&path, "author = \"yarrib\"\npython_version = \"3.12\"\n").unwrap();
        let standards = load_standards(Some(&path)).unwrap();
        assert_eq!(standards.get("author").unwrap(), "yarrib");
        assert_eq!(standards.get("python_version").unwrap(), "3.12");
    }
}
