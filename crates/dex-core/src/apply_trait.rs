//! Apply a trait to an existing project directory.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::error::{DexError, TemplateError};
use crate::template::engine::TemplateEngine;
use crate::traits::{ConflictPolicy, Trait, TraitFileRule};

/// Result of a successful `dex add` operation.
#[derive(Debug)]
pub struct TraitResult {
    /// New files written to the project.
    pub files_created: Vec<PathBuf>,
    /// Existing files that were appended to (patches).
    pub files_patched: Vec<PathBuf>,
    /// Files skipped because they already existed and conflict policy was `skip`.
    pub files_skipped: Vec<PathBuf>,
    pub directories_created: Vec<PathBuf>,
}

/// Apply a trait to `target_dir` using the given variable values.
///
/// # File conflicts
///
/// Each `[[files]]` rule carries a `conflict` policy:
/// - `error` (default) — abort if the destination file already exists.
/// - `overwrite` — replace the existing file.
/// - `skip` — leave the existing file and continue.
///
/// # Patches
///
/// `[[patches]]` rules append rendered content to an existing file. If the
/// target file does not yet exist, it is created.
pub fn apply_trait(
    t: &Trait,
    target_dir: &Path,
    variables: &HashMap<String, minijinja::Value>,
) -> Result<TraitResult, DexError> {
    let engine = TemplateEngine::new();
    let context = minijinja::Value::from_serialize(variables);

    let mut files_created = Vec::new();
    let mut files_patched = Vec::new();
    let mut files_skipped = Vec::new();
    let mut directories_created = Vec::new();

    // `dex add` operates on an existing project; create target_dir only as a fallback.
    if !target_dir.exists() {
        std::fs::create_dir_all(target_dir).map_err(|source| DexError::Io {
            path: target_dir.to_path_buf(),
            source,
        })?;
        directories_created.push(target_dir.to_path_buf());
    }

    // --- Write files ---
    for (rel_path, content) in &t.files {
        if !should_include_file(rel_path, &t.file_rules, variables) {
            continue;
        }

        // Render the path itself (supports `{{ var }}` in directory/file names).
        let rendered_path_str = engine.render_path(&rel_path.to_string_lossy(), &context)?;
        let rendered_path = PathBuf::from(&rendered_path_str);

        // Strip `.j2` extension.
        let final_path = if rendered_path.extension().and_then(|e| e.to_str()) == Some("j2") {
            rendered_path.with_extension("")
        } else {
            rendered_path
        };

        let dest = target_dir.join(&final_path);

        // Apply conflict policy when destination already exists.
        if dest.exists() {
            let policy = get_conflict_policy(rel_path, &t.file_rules);
            match policy {
                ConflictPolicy::Error => {
                    return Err(DexError::Template(TemplateError::InvalidManifest(format!(
                        "file already exists: {} — use a different conflict policy or remove the file first",
                        dest.display()
                    ))));
                }
                ConflictPolicy::Skip => {
                    files_skipped.push(final_path);
                    continue;
                }
                ConflictPolicy::Overwrite => {} // fall through to write
            }
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

        // Render content if it's a Jinja2 template.
        let is_template = rel_path.extension().and_then(|e| e.to_str()) == Some("j2");
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

    // --- Apply patches ---
    for patch in &t.patches {
        // Check optional condition.
        if let Some(condition) = &patch.condition {
            let is_truthy = variables
                .get(condition)
                .map(|v| v.is_true())
                .unwrap_or(false);
            if !is_truthy {
                continue;
            }
        }

        let target_path = target_dir.join(&patch.target);

        // Render the append content.
        let rendered_append = engine.render_string(&patch.append, &context)?;

        // Read existing content (create the file if missing).
        let mut existing = if target_path.exists() {
            std::fs::read_to_string(&target_path).map_err(|source| DexError::Io {
                path: target_path.clone(),
                source,
            })?
        } else {
            String::new()
        };

        // Ensure a trailing newline before appending.
        if !existing.is_empty() && !existing.ends_with('\n') {
            existing.push('\n');
        }
        existing.push_str(&rendered_append);

        std::fs::write(&target_path, &existing).map_err(|source| DexError::Io {
            path: target_path.clone(),
            source,
        })?;
        files_patched.push(PathBuf::from(&patch.target));
    }

    Ok(TraitResult {
        files_created,
        files_patched,
        files_skipped,
        directories_created,
    })
}

/// Check whether a file should be included based on file rules and variable values.
fn should_include_file(
    rel_path: &Path,
    file_rules: &[TraitFileRule],
    variables: &HashMap<String, minijinja::Value>,
) -> bool {
    for rule in file_rules {
        let rule_src = Path::new(&rule.src);
        if rel_path.starts_with(rule_src) || rel_path == rule_src {
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

/// Return the conflict policy for a given file path.
fn get_conflict_policy(rel_path: &Path, file_rules: &[TraitFileRule]) -> ConflictPolicy {
    for rule in file_rules {
        let rule_src = Path::new(&rule.src);
        if rel_path.starts_with(rule_src) || rel_path == rule_src {
            return rule.conflict.clone();
        }
    }
    ConflictPolicy::Error
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::{PatchRule, Trait, TraitFileRule, TraitMeta};

    fn make_trait(
        files: HashMap<PathBuf, String>,
        file_rules: Vec<TraitFileRule>,
        patches: Vec<PatchRule>,
    ) -> Trait {
        Trait {
            meta: TraitMeta {
                name: "test".into(),
                description: "Test trait".into(),
                version: "0.1.0".into(),
                min_dex_version: None,
            },
            variables: vec![],
            file_rules,
            files,
            patches,
        }
    }

    #[test]
    fn apply_trait_writes_new_files() {
        let dir = tempfile::tempdir().unwrap();
        let mut files = HashMap::new();
        files.insert(PathBuf::from("Dockerfile.j2"), "FROM python:{{ py }}".to_string());

        let t = make_trait(files, vec![], vec![]);
        let mut vars = HashMap::new();
        vars.insert("py".to_string(), minijinja::Value::from("3.12-slim"));

        let result = apply_trait(&t, dir.path(), &vars).unwrap();
        assert_eq!(result.files_created.len(), 1);

        let content = std::fs::read_to_string(dir.path().join("Dockerfile")).unwrap();
        assert_eq!(content, "FROM python:3.12-slim");
    }

    #[test]
    fn apply_trait_conflict_error_aborts() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("Dockerfile"), "existing").unwrap();

        let mut files = HashMap::new();
        files.insert(PathBuf::from("Dockerfile"), "new".to_string());

        let rules = vec![TraitFileRule {
            src: "Dockerfile".to_string(),
            dest: None,
            condition: None,
            conflict: ConflictPolicy::Error,
        }];

        let t = make_trait(files, rules, vec![]);
        let result = apply_trait(&t, dir.path(), &HashMap::new());
        assert!(result.is_err());
    }

    #[test]
    fn apply_trait_conflict_skip_preserves_existing() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("Dockerfile"), "original").unwrap();

        let mut files = HashMap::new();
        files.insert(PathBuf::from("Dockerfile"), "new".to_string());

        let rules = vec![TraitFileRule {
            src: "Dockerfile".to_string(),
            dest: None,
            condition: None,
            conflict: ConflictPolicy::Skip,
        }];

        let t = make_trait(files, rules, vec![]);
        let result = apply_trait(&t, dir.path(), &HashMap::new()).unwrap();
        assert_eq!(result.files_skipped.len(), 1);
        assert_eq!(
            std::fs::read_to_string(dir.path().join("Dockerfile")).unwrap(),
            "original"
        );
    }

    #[test]
    fn apply_trait_conflict_overwrite_replaces() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("Dockerfile"), "original").unwrap();

        let mut files = HashMap::new();
        files.insert(PathBuf::from("Dockerfile"), "new".to_string());

        let rules = vec![TraitFileRule {
            src: "Dockerfile".to_string(),
            dest: None,
            condition: None,
            conflict: ConflictPolicy::Overwrite,
        }];

        let t = make_trait(files, rules, vec![]);
        let result = apply_trait(&t, dir.path(), &HashMap::new()).unwrap();
        assert_eq!(result.files_created.len(), 1);
        assert_eq!(
            std::fs::read_to_string(dir.path().join("Dockerfile")).unwrap(),
            "new"
        );
    }

    #[test]
    fn apply_trait_patch_appends_to_existing_file() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("dex.toml"), "[project]\nname = \"foo\"\n").unwrap();

        let patches = vec![PatchRule {
            target: "dex.toml".to_string(),
            append: "\n[tasks.docker-build]\ncommand = \"docker build .\"\n".to_string(),
            condition: None,
        }];

        let t = make_trait(HashMap::new(), vec![], patches);
        let result = apply_trait(&t, dir.path(), &HashMap::new()).unwrap();
        assert_eq!(result.files_patched.len(), 1);

        let content = std::fs::read_to_string(dir.path().join("dex.toml")).unwrap();
        assert!(content.contains("[project]"));
        assert!(content.contains("[tasks.docker-build]"));
    }

    #[test]
    fn apply_trait_conditional_file_excluded_when_falsy() {
        let dir = tempfile::tempdir().unwrap();
        let mut files = HashMap::new();
        files.insert(PathBuf::from(".github/ci.yml"), "ci: true".to_string());

        let rules = vec![TraitFileRule {
            src: ".github/".to_string(),
            dest: None,
            condition: Some("include_ci".to_string()),
            conflict: ConflictPolicy::Error,
        }];

        let t = make_trait(files, rules, vec![]);
        let mut vars = HashMap::new();
        vars.insert("include_ci".to_string(), minijinja::Value::from(false));

        let result = apply_trait(&t, dir.path(), &vars).unwrap();
        assert!(result.files_created.is_empty());
    }
}
