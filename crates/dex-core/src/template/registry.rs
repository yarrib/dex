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

    Ok(Template {
        meta: manifest.meta(),
        variables: manifest.variables,
        file_rules: manifest.files,
        files,
    })
}

/// Recursively load all files from a template's `files/` directory.
fn load_template_files(dir: &Path) -> Result<HashMap<PathBuf, String>, DexError> {
    let mut files = HashMap::new();

    for entry in walkdir::WalkDir::new(dir).into_iter() {
        let entry = entry.map_err(|e| DexError::Io {
            path: dir.to_path_buf(),
            source: std::io::Error::new(std::io::ErrorKind::Other, e),
        })?;

        if entry.file_type().is_file() {
            let rel_path = entry
                .path()
                .strip_prefix(dir)
                .expect("walkdir entry should be under base dir");

            let content =
                std::fs::read_to_string(entry.path()).map_err(|source| DexError::Io {
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
        if manifest_path.is_file() {
            if let Ok(manifest) = TemplateManifest::from_path(&manifest_path) {
                templates.push(manifest.meta());
            }
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

    let manifest_file = template_dir
        .get_file(format!("{name}/template.toml"))
        .or_else(|| template_dir.get_file("template.toml"))
        .ok_or_else(|| {
            DexError::Template(TemplateError::InvalidManifest(format!(
                "no template.toml in embedded template '{name}'"
            )))
        })?;

    let manifest_str = manifest_file.contents_utf8().ok_or_else(|| {
        DexError::Template(TemplateError::InvalidManifest(
            "template.toml is not valid UTF-8".to_string(),
        ))
    })?;

    let manifest = TemplateManifest::from_str(manifest_str)?;

    // Collect files from the embedded "files/" subdirectory.
    let mut files = HashMap::new();
    let files_prefix = Path::new("files");

    fn collect_files(
        dir: &include_dir::Dir<'_>,
        base_prefix: &Path,
        files: &mut HashMap<PathBuf, String>,
    ) {
        for file in dir.files() {
            if let Some(rel) = file.path().strip_prefix(base_prefix).ok() {
                if let Some(content) = file.contents_utf8() {
                    files.insert(rel.to_path_buf(), content.to_string());
                }
            }
        }
        for subdir in dir.dirs() {
            collect_files(subdir, base_prefix, files);
        }
    }

    if let Some(files_dir) = template_dir.get_dir("files") {
        collect_files(files_dir, files_prefix, &mut files);
    }

    Ok(Template {
        meta: manifest.meta(),
        variables: manifest.variables,
        file_rules: manifest.files,
        files,
    })
}

fn list_embedded_templates() -> Result<Vec<TemplateMeta>, DexError> {
    let mut templates = Vec::new();

    for dir in EMBEDDED_TEMPLATES.dirs() {
        let manifest_file = dir.get_file("template.toml");
        if let Some(file) = manifest_file {
            if let Some(content) = file.contents_utf8() {
                if let Ok(manifest) = TemplateManifest::from_str(content) {
                    templates.push(manifest.meta());
                }
            }
        }
    }

    Ok(templates)
}
