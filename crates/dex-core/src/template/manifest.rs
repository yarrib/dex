//! Template manifest (`template.toml`) parsing.

use serde::Deserialize;
use std::path::Path;

use crate::error::{DexError, TemplateError};
use crate::template::{TemplateMeta, VariableSpec};

/// Raw deserialized template manifest from `template.toml`.
#[derive(Debug, Deserialize)]
pub struct TemplateManifest {
    pub template: TemplateMetaRaw,
    #[serde(default)]
    pub variables: Vec<VariableSpec>,
    #[serde(default)]
    pub files: Vec<FileRule>,
    #[serde(default)]
    pub hooks: Option<HooksSpec>,
}

/// Raw template metadata from `[template]` section.
#[derive(Debug, Deserialize)]
pub struct TemplateMetaRaw {
    pub name: String,
    pub description: String,
    pub version: String,
    #[serde(default)]
    pub min_dex_version: Option<String>,
}

/// Conditional file inclusion / path remapping rule.
#[derive(Debug, Deserialize)]
pub struct FileRule {
    pub src: String,
    #[serde(default)]
    pub dest: Option<String>,
    #[serde(default)]
    pub condition: Option<String>,
}

/// Hook script references.
#[derive(Debug, Deserialize)]
pub struct HooksSpec {
    #[serde(default)]
    pub pre_scaffold: Option<String>,
    #[serde(default)]
    pub post_scaffold: Option<String>,
}

impl TemplateManifest {
    /// Parse a `template.toml` file.
    pub fn from_path(path: &Path) -> Result<Self, DexError> {
        let content = std::fs::read_to_string(path).map_err(|source| DexError::Io {
            path: path.to_path_buf(),
            source,
        })?;
        Self::from_str(&content)
    }

    /// Parse a `template.toml` from a string.
    pub fn from_str(content: &str) -> Result<Self, DexError> {
        toml::from_str(content).map_err(|e| {
            DexError::Template(TemplateError::InvalidManifest(e.to_string()))
        })
    }

    /// Convert to a `TemplateMeta` for listing.
    #[must_use]
    pub fn meta(&self) -> TemplateMeta {
        TemplateMeta {
            name: self.template.name.clone(),
            description: self.template.description.clone(),
            version: self.template.version.clone(),
            min_dex_version: self.template.min_dex_version.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_minimal_manifest() {
        let toml_str = r#"
            [template]
            name = "default"
            description = "Default project template"
            version = "0.1.0"
        "#;
        let manifest = TemplateManifest::from_str(toml_str).unwrap();
        assert_eq!(manifest.template.name, "default");
        assert!(manifest.variables.is_empty());
        assert!(manifest.files.is_empty());
    }

    #[test]
    fn parse_full_manifest() {
        let toml_str = r#"
            [template]
            name = "ml-pipeline"
            description = "ML pipeline template"
            version = "0.1.0"
            min_dex_version = "0.1.0"

            [[variables]]
            name = "project_name"
            prompt = "Project name"
            type = "string"
            required = true
            validate = "^[a-z][a-z0-9_-]*$"

            [[variables]]
            name = "python_version"
            prompt = "Python version"
            type = "choice"
            choices = ["3.10", "3.11", "3.12"]
            default = "3.11"

            [[files]]
            src = ".github/"
            condition = "include_ci"

            [hooks]
            post_scaffold = "hooks/post_scaffold.py"
        "#;
        let manifest = TemplateManifest::from_str(toml_str).unwrap();
        assert_eq!(manifest.variables.len(), 2);
        assert_eq!(manifest.files.len(), 1);
        assert!(manifest.hooks.unwrap().post_scaffold.is_some());
    }
}
