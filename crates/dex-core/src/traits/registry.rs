//! Trait discovery and loading.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::error::{DexError, TemplateError};
use crate::traits::manifest::TraitManifest;
use crate::traits::{Trait, TraitMeta};

// Built-in traits are embedded at compile time.
static EMBEDDED_TRAITS: include_dir::Dir<'_> =
    include_dir::include_dir!("$CARGO_MANIFEST_DIR/../../traits");

/// Load a built-in or directory-based trait by name.
///
/// Resolution order:
/// 1. Embedded (compiled into binary)
/// 2. Directory path, if provided
pub fn load_trait(name: &str, extra_dir: Option<&Path>) -> Result<Trait, DexError> {
    // 1. Try extra directory first (higher priority).
    if let Some(dir) = extra_dir {
        let trait_dir = dir.join(name);
        if trait_dir.join("trait.toml").exists() {
            return load_directory_trait(&trait_dir);
        }
    }

    // 2. Embedded.
    load_embedded_trait(name)
}

/// List all available traits from embedded + optional extra directory.
pub fn list_traits(extra_dir: Option<&Path>) -> Result<Vec<TraitMeta>, DexError> {
    let mut traits: HashMap<String, TraitMeta> = HashMap::new();

    // Embedded traits (lowest priority).
    for meta in list_embedded_traits()? {
        traits.insert(meta.name.clone(), meta);
    }

    // Extra directory (higher priority — overrides embedded).
    if let Some(dir) = extra_dir {
        for meta in list_directory_traits(dir)? {
            traits.insert(meta.name.clone(), meta);
        }
    }

    let mut result: Vec<TraitMeta> = traits.into_values().collect();
    result.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(result)
}

// --- Embedded loading ---

fn load_embedded_trait(name: &str) -> Result<Trait, DexError> {
    let trait_dir = EMBEDDED_TRAITS
        .get_dir(name)
        .ok_or_else(|| DexError::Template(TemplateError::NotFound(name.to_string())))?;

    let manifest_path = trait_dir.path().join("trait.toml");
    let manifest_file = trait_dir.get_file(&manifest_path).ok_or_else(|| {
        DexError::Template(TemplateError::InvalidManifest(format!(
            "no trait.toml in embedded trait '{name}'"
        )))
    })?;

    let manifest_str = manifest_file.contents_utf8().ok_or_else(|| {
        DexError::Template(TemplateError::InvalidManifest(
            "trait.toml is not valid UTF-8".to_string(),
        ))
    })?;

    let manifest = TraitManifest::parse(manifest_str)?;

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

    let files_dir_path = trait_dir.path().join("files");
    if let Some(files_dir) = trait_dir.get_dir(&files_dir_path) {
        collect_files(files_dir, &files_dir_path, &mut files);
    }

    Ok(Trait {
        meta: manifest.meta(),
        variables: manifest.variables,
        file_rules: manifest.files,
        files,
        patches: manifest.patches,
    })
}

fn list_embedded_traits() -> Result<Vec<TraitMeta>, DexError> {
    let mut traits = Vec::new();

    for dir in EMBEDDED_TRAITS.dirs() {
        let manifest_path = dir.path().join("trait.toml");
        if let Some(file) = dir.get_file(&manifest_path)
            && let Some(content) = file.contents_utf8()
            && let Ok(manifest) = TraitManifest::parse(content)
        {
            traits.push(manifest.meta());
        }
    }

    Ok(traits)
}

// --- Directory loading ---

fn load_directory_trait(trait_dir: &Path) -> Result<Trait, DexError> {
    let manifest_path = trait_dir.join("trait.toml");
    let manifest = TraitManifest::from_path(&manifest_path)?;

    let files_dir = trait_dir.join("files");
    let files = if files_dir.is_dir() {
        load_trait_files(&files_dir)?
    } else {
        HashMap::new()
    };

    Ok(Trait {
        meta: manifest.meta(),
        variables: manifest.variables,
        file_rules: manifest.files,
        files,
        patches: manifest.patches,
    })
}

fn load_trait_files(dir: &Path) -> Result<HashMap<PathBuf, String>, DexError> {
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

fn list_directory_traits(base: &Path) -> Result<Vec<TraitMeta>, DexError> {
    let mut traits = Vec::new();

    if !base.is_dir() {
        return Ok(traits);
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

        let manifest_path = entry.path().join("trait.toml");
        if manifest_path.is_file()
            && let Ok(manifest) = TraitManifest::from_path(&manifest_path)
        {
            traits.push(manifest.meta());
        }
    }

    Ok(traits)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_traits_are_not_empty() {
        let traits = list_embedded_traits().unwrap();
        assert!(!traits.is_empty(), "no embedded traits found");
    }

    #[test]
    fn load_embedded_docker_trait() {
        let t = load_embedded_trait("docker").unwrap();
        assert_eq!(t.meta.name, "docker");
        assert!(!t.files.is_empty(), "docker trait should have files");
    }

    #[test]
    fn load_embedded_trait_not_found_returns_error() {
        let result = load_embedded_trait("does-not-exist");
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
    fn load_trait_from_directory() {
        let base = tempfile::tempdir().unwrap();
        let trait_dir = base.path().join("my-trait");
        let files_dir = trait_dir.join("files");
        std::fs::create_dir_all(&files_dir).unwrap();
        std::fs::write(
            trait_dir.join("trait.toml"),
            r#"[trait]
name = "my-trait"
description = "Test trait"
version = "0.1.0"
"#,
        )
        .unwrap();
        std::fs::write(files_dir.join("hello.txt"), "hello world").unwrap();

        let t = load_trait("my-trait", Some(base.path())).unwrap();
        assert_eq!(t.meta.name, "my-trait");
        assert_eq!(t.files.len(), 1);
    }
}
