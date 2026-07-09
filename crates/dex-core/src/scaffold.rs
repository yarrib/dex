//! Scaffolding: render a template into a target directory.

use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};

use crate::error::DexError;
use crate::template::engine::TemplateEngine;
use crate::template::{OnSuccessSpec, Template};

/// Result of a successful scaffold operation.
#[derive(Debug)]
pub struct ScaffoldResult {
    pub files_created: Vec<PathBuf>,
    pub directories_created: Vec<PathBuf>,
    /// Post-scaffold activation config from the template (if any).
    pub on_success: Option<OnSuccessSpec>,
}

/// A fully rendered template: final relative paths (variables interpolated,
/// `.j2` stripped) mapped to rendered content. Pure data — nothing on disk.
///
/// This is the unit `dex update` compares: rendering the same template at two
/// refs with the same answers yields two `RenderedTree`s to diff.
#[derive(Debug, Default, PartialEq)]
pub struct RenderedTree {
    pub files: BTreeMap<PathBuf, String>,
}

impl RenderedTree {
    /// Read a previously written tree back from a directory (e.g. the
    /// `.dex/cache/baseline/` written at init time).
    pub fn from_dir(dir: &Path) -> Result<Self, DexError> {
        let mut files = BTreeMap::new();

        if !dir.is_dir() {
            return Ok(Self { files });
        }

        for entry in walkdir::WalkDir::new(dir) {
            let entry = entry.map_err(|e| DexError::Io {
                path: dir.to_path_buf(),
                source: std::io::Error::other(e),
            })?;

            if entry.file_type().is_file() {
                let rel = entry
                    .path()
                    .strip_prefix(dir)
                    .map_err(|e| DexError::Io {
                        path: entry.path().to_path_buf(),
                        source: std::io::Error::other(e),
                    })?
                    .to_path_buf();

                let content =
                    std::fs::read_to_string(entry.path()).map_err(|source| DexError::Io {
                        path: entry.path().to_path_buf(),
                        source,
                    })?;

                files.insert(rel, content);
            }
        }

        Ok(Self { files })
    }
}

/// One rendered template file, keeping the source path so write policies
/// (file rules are keyed on source paths) can still be applied.
struct RenderedEntry {
    source: PathBuf,
    dest: PathBuf,
    content: String,
}

/// Render every included template file without touching the filesystem.
fn render_entries(
    template: &Template,
    variables: &HashMap<String, minijinja::Value>,
) -> Result<Vec<RenderedEntry>, DexError> {
    let engine = TemplateEngine::new();
    let context = minijinja::Value::from_serialize(variables);

    let mut entries = Vec::new();

    for (rel_path, content) in &template.files {
        // Check file rules for conditional inclusion.
        if !should_include_file(rel_path, &template.file_rules, variables) {
            continue;
        }

        // Render the file path (variable interpolation in directory/file names).
        let rendered_path_str = engine.render_path(&rel_path.to_string_lossy(), &context)?;
        let rendered_path = PathBuf::from(&rendered_path_str);

        // Strip `.j2` extension if present.
        let final_path = if rendered_path.extension().and_then(|e| e.to_str()) == Some("j2") {
            rendered_path.with_extension("")
        } else {
            rendered_path
        };

        // Render content through template engine if it's a .j2 file.
        let is_template = rel_path.extension().and_then(|e| e.to_str()) == Some("j2");

        let rendered_content = if is_template {
            engine.render_string(content, &context)?
        } else {
            content.clone()
        };

        entries.push(RenderedEntry {
            source: rel_path.clone(),
            dest: final_path,
            content: rendered_content,
        });
    }

    Ok(entries)
}

/// Render a template fully in memory with the given variables.
///
/// Applies `[[files]]` condition rules and path/content rendering exactly like
/// [`scaffold`], but performs no filesystem writes and no overwrite checks.
pub fn render_tree(
    template: &Template,
    variables: &HashMap<String, minijinja::Value>,
) -> Result<RenderedTree, DexError> {
    let files = render_entries(template, variables)?
        .into_iter()
        .map(|e| (e.dest, e.content))
        .collect();
    Ok(RenderedTree { files })
}

/// Write a rendered tree into a directory, creating parents as needed.
/// Existing files are overwritten unconditionally. Returns the relative
/// paths written.
pub fn write_tree(tree: &RenderedTree, target_dir: &Path) -> Result<Vec<PathBuf>, DexError> {
    if !target_dir.exists() {
        std::fs::create_dir_all(target_dir).map_err(|source| DexError::Io {
            path: target_dir.to_path_buf(),
            source,
        })?;
    }

    let mut written = Vec::new();

    for (rel_path, content) in &tree.files {
        let dest = target_dir.join(rel_path);

        if let Some(parent) = dest.parent()
            && !parent.exists()
        {
            std::fs::create_dir_all(parent).map_err(|source| DexError::Io {
                path: parent.to_path_buf(),
                source,
            })?;
        }

        std::fs::write(&dest, content).map_err(|source| DexError::Io {
            path: dest.clone(),
            source,
        })?;
        written.push(rel_path.clone());
    }

    Ok(written)
}

/// Scaffold a project from a template into a target directory.
///
/// Renders all template files through the Jinja2 engine with the given variables,
/// writing the results to `target_dir`. File paths containing `{{ var }}` syntax
/// are also rendered (variable interpolation in paths).
pub fn scaffold(
    template: &Template,
    target_dir: &Path,
    variables: &HashMap<String, minijinja::Value>,
) -> Result<ScaffoldResult, DexError> {
    let mut files_created = Vec::new();
    let mut directories_created = Vec::new();

    // Create the target directory if it doesn't exist.
    if !target_dir.exists() {
        std::fs::create_dir_all(target_dir).map_err(|source| DexError::Io {
            path: target_dir.to_path_buf(),
            source,
        })?;
        directories_created.push(target_dir.to_path_buf());
    }

    for entry in render_entries(template, variables)? {
        let dest = target_dir.join(&entry.dest);

        // Respect file rule overwrite flag: skip if file exists and rule says don't overwrite.
        if dest.exists() && !get_overwrite_flag(&entry.source, &template.file_rules) {
            continue;
        }

        // Create parent directories.
        if let Some(parent) = dest.parent()
            && !parent.exists()
        {
            std::fs::create_dir_all(parent).map_err(|source| DexError::Io {
                path: parent.to_path_buf(),
                source,
            })?;
            directories_created.push(parent.to_path_buf());
        }

        std::fs::write(&dest, &entry.content).map_err(|source| DexError::Io {
            path: dest.clone(),
            source,
        })?;
        files_created.push(entry.dest);
    }

    Ok(ScaffoldResult {
        files_created,
        directories_created,
        on_success: template.on_success.clone(),
    })
}

/// Return the effective overwrite flag for a file path based on file rules.
///
/// Returns `true` (overwrite) if no rule matches. When a rule matches, returns
/// that rule's `overwrite` field.
fn get_overwrite_flag(rel_path: &Path, file_rules: &[crate::template::FileRule]) -> bool {
    for rule in file_rules {
        let rule_src = Path::new(&rule.src);
        if rel_path.starts_with(rule_src) {
            return rule.overwrite;
        }
    }
    true
}

/// Check whether a file should be included based on file rules and variable values.
fn should_include_file(
    rel_path: &Path,
    file_rules: &[crate::template::FileRule],
    variables: &HashMap<String, minijinja::Value>,
) -> bool {
    for rule in file_rules {
        let rule_src = Path::new(&rule.src);

        // Check if this rule applies to the file.
        if rel_path.starts_with(rule_src)
            && let Some(condition) = &rule.condition
        {
            let is_truthy = variables
                .get(condition)
                .map(|v| v.is_true())
                .unwrap_or(false);

            if !is_truthy {
                return false;
            }
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::template::{FileRule, TemplateMeta};

    fn make_template(files: HashMap<PathBuf, String>, file_rules: Vec<FileRule>) -> Template {
        Template {
            meta: TemplateMeta {
                name: "test".into(),
                description: "Test template".into(),
                version: "0.1.0".into(),
                min_dex_version: None,
            },
            variables: vec![],
            file_rules,
            files,
            suggested_skills: vec![],
            on_success: None,
            hooks: None,
        }
    }

    #[test]
    fn scaffold_simple_template() {
        let dir = tempfile::tempdir().unwrap();
        let mut files = HashMap::new();
        files.insert(PathBuf::from("README.md"), "# Hello".to_string());
        files.insert(
            PathBuf::from("src/main.py.j2"),
            "# Project: {{ project_name }}".to_string(),
        );

        let template = make_template(files, vec![]);
        let mut vars = HashMap::new();
        vars.insert(
            "project_name".to_string(),
            minijinja::Value::from("my_project"),
        );

        let result = scaffold(&template, dir.path(), &vars).unwrap();
        assert_eq!(result.files_created.len(), 2);

        // Check rendered content
        let main_content = std::fs::read_to_string(dir.path().join("src/main.py")).unwrap();
        assert_eq!(main_content, "# Project: my_project");

        // Check non-template file is copied verbatim
        let readme_content = std::fs::read_to_string(dir.path().join("README.md")).unwrap();
        assert_eq!(readme_content, "# Hello");
    }

    #[test]
    fn scaffold_overwrite_false_skips_existing_file() {
        let dir = tempfile::tempdir().unwrap();
        let existing = dir.path().join("config.toml");
        std::fs::write(&existing, "original").unwrap();

        let mut files = HashMap::new();
        files.insert(PathBuf::from("config.toml"), "overwritten".to_string());

        let rules = vec![FileRule {
            src: "config.toml".to_string(),
            dest: None,
            condition: None,
            overwrite: false,
            context_role: None,
            context_description: None,
        }];

        let template = make_template(files, rules);
        let result = scaffold(&template, dir.path(), &HashMap::new()).unwrap();

        // File was skipped, so not in files_created
        assert!(result.files_created.is_empty());
        // Original content preserved
        assert_eq!(std::fs::read_to_string(&existing).unwrap(), "original");
    }

    #[test]
    fn scaffold_conditional_exclusion() {
        let dir = tempfile::tempdir().unwrap();
        let mut files = HashMap::new();
        files.insert(PathBuf::from("README.md"), "# Hello".to_string());
        files.insert(PathBuf::from(".github/ci.yml"), "name: CI".to_string());

        let rules = vec![FileRule {
            src: ".github/".to_string(),
            dest: None,
            condition: Some("include_ci".to_string()),
            overwrite: false,
            context_role: None,
            context_description: None,
        }];

        let template = make_template(files, rules);
        let mut vars = HashMap::new();
        vars.insert("include_ci".to_string(), minijinja::Value::from(false));

        let result = scaffold(&template, dir.path(), &vars).unwrap();
        assert_eq!(result.files_created.len(), 1);
        assert!(!dir.path().join(".github/ci.yml").exists());
    }

    #[test]
    fn render_tree_matches_scaffold_output() {
        let dir = tempfile::tempdir().unwrap();
        let mut files = HashMap::new();
        files.insert(PathBuf::from("README.md"), "# Hello".to_string());
        files.insert(
            PathBuf::from("src/{{ project_name }}/main.py.j2"),
            "# Project: {{ project_name }}".to_string(),
        );

        let template = make_template(files, vec![]);
        let mut vars = HashMap::new();
        vars.insert("project_name".to_string(), minijinja::Value::from("proj"));

        let tree = render_tree(&template, &vars).unwrap();
        assert_eq!(tree.files.len(), 2);
        assert_eq!(
            tree.files.get(Path::new("src/proj/main.py")).unwrap(),
            "# Project: proj"
        );

        scaffold(&template, dir.path(), &vars).unwrap();
        for (rel, content) in &tree.files {
            assert_eq!(
                &std::fs::read_to_string(dir.path().join(rel)).unwrap(),
                content
            );
        }
    }

    #[test]
    fn render_tree_applies_condition_rules() {
        let mut files = HashMap::new();
        files.insert(PathBuf::from(".github/ci.yml"), "name: CI".to_string());
        files.insert(PathBuf::from("README.md"), "# Hello".to_string());

        let rules = vec![FileRule {
            src: ".github/".to_string(),
            dest: None,
            condition: Some("include_ci".to_string()),
            overwrite: false,
            context_role: None,
            context_description: None,
        }];

        let template = make_template(files, rules);
        let mut vars = HashMap::new();
        vars.insert("include_ci".to_string(), minijinja::Value::from(false));

        let tree = render_tree(&template, &vars).unwrap();
        assert_eq!(tree.files.len(), 1);
        assert!(tree.files.contains_key(Path::new("README.md")));
    }

    #[test]
    fn write_tree_and_from_dir_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let mut files = BTreeMap::new();
        files.insert(PathBuf::from("a.txt"), "alpha\n".to_string());
        files.insert(PathBuf::from("nested/b.txt"), "beta\n".to_string());
        let tree = RenderedTree { files };

        let written = write_tree(&tree, dir.path()).unwrap();
        assert_eq!(written.len(), 2);

        let read_back = RenderedTree::from_dir(dir.path()).unwrap();
        assert_eq!(read_back, tree);
    }

    #[test]
    fn from_dir_missing_directory_is_empty() {
        let dir = tempfile::tempdir().unwrap();
        let tree = RenderedTree::from_dir(&dir.path().join("nope")).unwrap();
        assert!(tree.files.is_empty());
    }
}
