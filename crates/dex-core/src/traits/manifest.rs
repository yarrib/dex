//! Trait manifest (`trait.toml`) parsing.

use serde::Deserialize;
use std::path::Path;

use crate::error::{DexError, TemplateError};
use crate::template::variables::VariableSpec;

/// Raw deserialized trait manifest from `trait.toml`.
#[derive(Debug, Deserialize)]
pub struct TraitManifest {
    #[serde(rename = "trait")]
    pub meta: TraitMetaRaw,
    #[serde(default)]
    pub variables: Vec<VariableSpec>,
    #[serde(default)]
    pub files: Vec<TraitFileRule>,
    #[serde(default)]
    pub patches: Vec<PatchRule>,
}

/// Raw trait metadata from `[trait]` section.
#[derive(Debug, Deserialize)]
pub struct TraitMetaRaw {
    pub name: String,
    pub description: String,
    pub version: String,
    #[serde(default)]
    pub min_dex_version: Option<String>,
}

/// What to do when a trait file already exists in the target project.
#[derive(Debug, Default, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ConflictPolicy {
    /// Abort the operation (default — safe).
    #[default]
    Error,
    /// Replace the existing file.
    Overwrite,
    /// Leave the existing file unchanged and continue.
    Skip,
}

/// Conditional file inclusion rule for a trait.
#[derive(Debug, Deserialize, Clone)]
pub struct TraitFileRule {
    /// Path prefix to match (e.g., `.github/` or `Dockerfile`).
    pub src: String,
    /// Optional destination path override (unused in v1, reserved).
    #[serde(default)]
    pub dest: Option<String>,
    /// Variable name: only include these files if the variable is truthy.
    #[serde(default)]
    pub condition: Option<String>,
    /// What to do if the destination file already exists.
    #[serde(default)]
    pub conflict: ConflictPolicy,
}

/// Append content to an existing file in the target project.
#[derive(Debug, Deserialize, Clone)]
pub struct PatchRule {
    /// Relative path of the file to patch (e.g., `dex.toml`).
    pub target: String,
    /// Content to append. May contain Jinja2 expressions.
    pub append: String,
    /// Optional variable condition; patch is skipped when falsy.
    #[serde(default)]
    pub condition: Option<String>,
}

impl TraitManifest {
    /// Parse a `trait.toml` file.
    pub fn from_path(path: &Path) -> Result<Self, DexError> {
        let content = std::fs::read_to_string(path).map_err(|source| DexError::Io {
            path: path.to_path_buf(),
            source,
        })?;
        Self::parse(&content)
    }

    /// Parse a `trait.toml` from a string.
    pub fn parse(content: &str) -> Result<Self, DexError> {
        toml::from_str(content)
            .map_err(|e| DexError::Template(TemplateError::InvalidManifest(e.to_string())))
    }

    /// Convert to a `TraitMeta` for listing.
    #[must_use]
    pub fn meta(&self) -> crate::traits::TraitMeta {
        crate::traits::TraitMeta {
            name: self.meta.name.clone(),
            description: self.meta.description.clone(),
            version: self.meta.version.clone(),
            min_dex_version: self.meta.min_dex_version.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_minimal_trait_manifest() {
        let toml_str = r#"
            [trait]
            name = "docker"
            description = "Add Docker support"
            version = "0.1.0"
        "#;
        let manifest = TraitManifest::parse(toml_str).unwrap();
        assert_eq!(manifest.meta.name, "docker");
        assert!(manifest.variables.is_empty());
        assert!(manifest.files.is_empty());
        assert!(manifest.patches.is_empty());
    }

    #[test]
    fn parse_trait_manifest_with_files_and_patches() {
        let toml_str = r#"
            [trait]
            name = "docker"
            description = "Add Docker support"
            version = "0.1.0"

            [[variables]]
            name = "base_image"
            prompt = "Base Docker image"
            type = "string"
            default = "python:3.12-slim"

            [[files]]
            src = "Dockerfile"
            conflict = "error"

            [[files]]
            src = ".dockerignore"
            conflict = "skip"

            [[patches]]
            target = "dex.toml"
            append = """
[tasks.docker-build]
command = "docker build -t myapp ."
description = "Build Docker image"
"""
        "#;
        let manifest = TraitManifest::parse(toml_str).unwrap();
        assert_eq!(manifest.meta.name, "docker");
        assert_eq!(manifest.variables.len(), 1);
        assert_eq!(manifest.files.len(), 2);
        assert_eq!(manifest.files[0].conflict, ConflictPolicy::Error);
        assert_eq!(manifest.files[1].conflict, ConflictPolicy::Skip);
        assert_eq!(manifest.patches.len(), 1);
        assert_eq!(manifest.patches[0].target, "dex.toml");
    }

    #[test]
    fn parse_trait_manifest_with_condition() {
        let toml_str = r#"
            [trait]
            name = "ci-github"
            description = "Add GitHub Actions CI"
            version = "0.1.0"

            [[files]]
            src = ".github/"
            condition = "include_ci"
            conflict = "error"
        "#;
        let manifest = TraitManifest::parse(toml_str).unwrap();
        assert_eq!(manifest.files[0].condition.as_deref(), Some("include_ci"));
    }
}
