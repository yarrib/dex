//! Template discovery and loading from various sources.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::error::{DexError, TemplateError};
use crate::template::manifest::TemplateManifest;
use crate::template::{Template, TemplateMeta};

/// Where templates can be loaded from.
#[derive(Debug, Clone)]
pub enum TemplateSource {
    /// Built-in templates embedded in the binary.
    Embedded,
    /// A directory on the filesystem containing template directories.
    Directory(PathBuf),
}

/// Load a template by name from the given source.
pub fn load_template(source: &TemplateSource, name: &str) -> Result<Template, DexError> {
    match source {
        TemplateSource::Embedded => load_embedded_template(name),
        TemplateSource::Directory(base) => load_directory_template(&base.join(name)),
    }
}

/// List all available templates from the given source.
pub fn list_templates(source: &TemplateSource) -> Result<Vec<TemplateMeta>, DexError> {
    match source {
        TemplateSource::Embedded => list_embedded_templates(),
        TemplateSource::Directory(base) => list_directory_templates(base),
    }
}

/// Load a template from a filesystem directory.
fn load_directory_template(template_dir: &Path) -> Result<Template, DexError> {
    let manifest_path = template_dir.join("template.toml");
    let manifest = TemplateManifest::from_path(&manifest_path)?;

    let files_dir = template_dir.join("files");
    let files = if files_dir.is_dir() {
        load_template_files(&files_dir)?
    } else {
        HashMap::new()
    };

    let suggested_skills = manifest
        .skills
        .as_ref()
        .map(|s| s.packs.clone())
        .unwrap_or_default();

    Ok(Template {
        meta: manifest.meta(),
        variables: manifest.variables(),
        file_rules: manifest.files,
        files,
        suggested_skills,
        on_success: manifest.on_success,
        hooks: manifest.hooks,
    })
}

/// Recursively load all files from a template's `files/` directory.
fn load_template_files(dir: &Path) -> Result<HashMap<PathBuf, String>, DexError> {
    let mut files = HashMap::new();

    for entry in walkdir::WalkDir::new(dir).into_iter() {
        let entry = entry.map_err(|e| DexError::Io {
            path: dir.to_path_buf(),
            source: std::io::Error::other(e),
        })?;

        if entry.file_type().is_file() {
            let rel_path = entry
                .path()
                .strip_prefix(dir)
                .expect("walkdir entry should be under base dir");

            let content = std::fs::read_to_string(entry.path()).map_err(|source| DexError::Io {
                path: entry.path().to_path_buf(),
                source,
            })?;

            files.insert(rel_path.to_path_buf(), content);
        }
    }

    Ok(files)
}

/// List templates from filesystem directories.
fn list_directory_templates(base: &Path) -> Result<Vec<TemplateMeta>, DexError> {
    let mut templates = Vec::new();

    if !base.is_dir() {
        return Ok(templates);
    }

    let entries = std::fs::read_dir(base).map_err(|source| DexError::Io {
        path: base.to_path_buf(),
        source,
    })?;

    for entry in entries {
        let entry = entry.map_err(|source| DexError::Io {
            path: base.to_path_buf(),
            source,
        })?;

        let manifest_path = entry.path().join("template.toml");
        if manifest_path.is_file()
            && let Ok(manifest) = TemplateManifest::from_path(&manifest_path)
        {
            templates.push(manifest.meta());
        }
    }

    Ok(templates)
}

// --- Embedded templates ---

// Built-in templates are embedded at compile time.
static EMBEDDED_TEMPLATES: include_dir::Dir<'_> =
    include_dir::include_dir!("$CARGO_MANIFEST_DIR/../../templates");

fn load_embedded_template(name: &str) -> Result<Template, DexError> {
    let template_dir = EMBEDDED_TEMPLATES
        .get_dir(name)
        .ok_or_else(|| DexError::Template(TemplateError::NotFound(name.to_string())))?;

    let manifest_path = template_dir.path().join("template.toml");
    let manifest_file = template_dir.get_file(&manifest_path).ok_or_else(|| {
        DexError::Template(TemplateError::InvalidManifest(format!(
            "no template.toml in embedded template '{name}'"
        )))
    })?;

    let manifest_str = manifest_file.contents_utf8().ok_or_else(|| {
        DexError::Template(TemplateError::InvalidManifest(
            "template.toml is not valid UTF-8".to_string(),
        ))
    })?;

    let manifest = TemplateManifest::parse(manifest_str)?;

    // Collect files from the embedded "files/" subdirectory.
    let mut files = HashMap::new();

    fn collect_files(
        dir: &include_dir::Dir<'_>,
        base_prefix: &Path,
        files: &mut HashMap<PathBuf, String>,
    ) {
        for file in dir.files() {
            if let Ok(rel) = file.path().strip_prefix(base_prefix)
                && let Some(content) = file.contents_utf8()
            {
                files.insert(rel.to_path_buf(), content.to_string());
            }
        }
        for subdir in dir.dirs() {
            collect_files(subdir, base_prefix, files);
        }
    }

    // base_prefix must be the full embedded path of the "files/" dir so that
    // strip_prefix produces paths relative to that dir (e.g. "src/main.py").
    let files_dir_path = template_dir.path().join("files");
    if let Some(files_dir) = template_dir.get_dir(&files_dir_path) {
        collect_files(files_dir, &files_dir_path, &mut files);
    }

    let suggested_skills = manifest
        .skills
        .as_ref()
        .map(|s| s.packs.clone())
        .unwrap_or_default();

    Ok(Template {
        meta: manifest.meta(),
        variables: manifest.variables(),
        file_rules: manifest.files,
        files,
        suggested_skills,
        on_success: manifest.on_success,
        hooks: manifest.hooks,
    })
}

fn list_embedded_templates() -> Result<Vec<TemplateMeta>, DexError> {
    let mut templates = Vec::new();

    for dir in EMBEDDED_TEMPLATES.dirs() {
        let manifest_path = dir.path().join("template.toml");
        let manifest_file = dir.get_file(&manifest_path);
        if let Some(file) = manifest_file
            && let Some(content) = file.contents_utf8()
            && let Ok(manifest) = TemplateManifest::parse(content)
        {
            templates.push(manifest.meta());
        }
    }

    Ok(templates)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_embedded_template_not_found_returns_error() {
        let result = load_embedded_template("does-not-exist");
        assert!(
            matches!(
                result,
                Err(crate::error::DexError::Template(
                    crate::error::TemplateError::NotFound(_)
                ))
            ),
            "expected TemplateError::NotFound"
        );
    }

    #[test]
    fn load_template_from_directory_source() {
        let base = tempfile::tempdir().unwrap();
        let tmpl_dir = base.path().join("my-tmpl");
        let files_dir = tmpl_dir.join("files");
        std::fs::create_dir_all(&files_dir).unwrap();
        std::fs::write(
            tmpl_dir.join("template.toml"),
            r#"[template]
name = "my-tmpl"
description = "Test"
version = "0.1.0"
"#,
        )
        .unwrap();
        std::fs::write(files_dir.join("hello.txt"), "hello world").unwrap();

        let source = TemplateSource::Directory(base.path().to_path_buf());
        let template = load_template(&source, "my-tmpl").unwrap();

        assert_eq!(template.meta.name, "my-tmpl");
        assert_eq!(template.files.len(), 1);
        assert!(
            template
                .files
                .contains_key(std::path::Path::new("hello.txt"))
        );
    }

    #[test]
    fn embedded_templates_are_not_empty() {
        let templates = list_embedded_templates().unwrap();
        assert!(!templates.is_empty(), "no embedded templates found");
        assert!(
            templates.iter().any(|t| t.name == "default"),
            "missing 'default' template"
        );
    }

    #[test]
    fn load_embedded_dabs_package_has_all_variables() {
        let template = load_embedded_template("dabs-package").unwrap();
        let names: Vec<&str> = template.variables.iter().map(|v| v.name.as_str()).collect();
        assert!(
            names.contains(&"project_name"),
            "missing project_name variable"
        );
        assert!(
            names.contains(&"python_version"),
            "missing python_version variable"
        );
        assert!(
            names.contains(&"include_notebook"),
            "missing include_notebook variable"
        );
        assert!(
            names.contains(&"include_job"),
            "missing include_job variable"
        );
        assert!(
            names.contains(&"use_serverless"),
            "missing use_serverless variable"
        );
        assert_eq!(
            template.variables.len(),
            5,
            "expected 5 variables, got: {names:?}"
        );
    }

    #[test]
    fn load_embedded_dabs_package_has_files() {
        let template = load_embedded_template("dabs-package").unwrap();
        assert!(
            !template.files.is_empty(),
            "dabs-package embedded template has no files"
        );
        assert!(
            template
                .files
                .keys()
                .any(|p| p.to_string_lossy().contains("pyproject.toml")),
            "missing pyproject.toml in embedded dabs-package files"
        );
    }
}
