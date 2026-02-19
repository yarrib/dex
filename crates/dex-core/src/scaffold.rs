//! Scaffolding: render a template into a target directory.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::error::DexError;
use crate::template::engine::TemplateEngine;
use crate::template::Template;

/// Result of a successful scaffold operation.
#[derive(Debug)]
pub struct ScaffoldResult {
    pub files_created: Vec<PathBuf>,
    pub directories_created: Vec<PathBuf>,
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
    let engine = TemplateEngine::new();
    let context = minijinja::Value::from_serialize(variables);

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

        let dest = target_dir.join(&final_path);

        // Create parent directories.
        if let Some(parent) = dest.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent).map_err(|source| DexError::Io {
                    path: parent.to_path_buf(),
                    source,
                })?;
                directories_created.push(parent.to_path_buf());
            }
        }

        // Render content through template engine if it's a .j2 file.
        let is_template = rel_path
            .extension()
            .and_then(|e| e.to_str())
            == Some("j2");

        let rendered_content = if is_template {
            engine.render_string(content, &context)?
        } else {
            content.clone()
        };

        std::fs::write(&dest, &rendered_content).map_err(|source| DexError::Io {
            path: dest.clone(),
            source,
        })?;
        files_created.push(final_path);
    }

    Ok(ScaffoldResult {
        files_created,
        directories_created,
    })
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
        if rel_path.starts_with(rule_src) {
            // If there's a condition, check the variable value.
            if let Some(condition) = &rule.condition {
                let is_truthy = variables
                    .get(condition)
                    .map(|v| v.is_true())
                    .unwrap_or(false);

                if !is_truthy {
                    return false;
                }
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
    fn scaffold_conditional_exclusion() {
        let dir = tempfile::tempdir().unwrap();
        let mut files = HashMap::new();
        files.insert(PathBuf::from("README.md"), "# Hello".to_string());
        files.insert(
            PathBuf::from(".github/ci.yml"),
            "name: CI".to_string(),
        );

        let rules = vec![FileRule {
            src: ".github/".to_string(),
            dest: None,
            condition: Some("include_ci".to_string()),
        }];

        let template = make_template(files, rules);
        let mut vars = HashMap::new();
        vars.insert("include_ci".to_string(), minijinja::Value::from(false));

        let result = scaffold(&template, dir.path(), &vars).unwrap();
        assert_eq!(result.files_created.len(), 1);
        assert!(!dir.path().join(".github/ci.yml").exists());
    }
}
