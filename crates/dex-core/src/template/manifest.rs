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
    /// Optional DABs template base. When present, `dex init` delegates to
    /// `databricks bundle init <source>` before rendering dex's own files.
    #[serde(default)]
    pub dabs: Option<DabsBaseSpec>,
}

/// DABs template base configuration from `[template.dabs]`.
///
/// Specifies a Databricks Asset Bundle template to use as the foundation
/// for scaffolding. The source can be any valid `databricks bundle init` target:
/// a Git URL, a local path, or a built-in DABs template name.
#[derive(Debug, Deserialize)]
pub struct DabsBaseSpec {
    /// Source for `databricks bundle init` — URL, local path, or built-in name.
    pub source: String,
    /// Maps dex variable names to DABs template variable names.
    /// Used to write a config JSON file for non-interactive DABs init.
    #[serde(default)]
    pub variable_map: std::collections::HashMap<String, String>,
}

/// Conditional file inclusion / path remapping rule.
#[derive(Debug, Deserialize)]
pub struct FileRule {
    pub src: String,
    #[serde(default)]
    pub dest: Option<String>,
    #[serde(default)]
    pub condition: Option<String>,
    /// When true, overwrite files that already exist in the target directory.
    /// Relevant for DABs-composite templates where the DABs scaffold may have
    /// created files that the dex layer wants to replace.
    #[serde(default)]
    pub overwrite: bool,
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
    fn parse_dabs_composite_manifest() {
        let toml_str = r#"
            [template]
            name = "ml-pipeline"
            description = "ML pipeline with DABs base"
            version = "0.1.0"

            [template.dabs]
            source = "https://github.com/databricks/bundle-examples/tree/main/default-python"

            [template.dabs.variable_map]
            project_name = "project_name"

            [[variables]]
            name = "project_name"
            prompt = "Project name"
            type = "string"
            required = true

            [[variables]]
            name = "include_ci"
            prompt = "Include CI?"
            type = "bool"
            default = true

            [[files]]
            src = ".github/"
            condition = "include_ci"
        "#;
        let manifest = TemplateManifest::from_str(toml_str).unwrap();
        let dabs = manifest.template.dabs.unwrap();
        assert!(dabs.source.contains("bundle-examples"));
        assert_eq!(dabs.variable_map.get("project_name").unwrap(), "project_name");
        assert_eq!(manifest.variables.len(), 2);
    }

    #[test]
    fn parse_standalone_manifest_has_no_dabs() {
        let toml_str = r#"
            [template]
            name = "default"
            description = "Standalone template"
            version = "0.1.0"
        "#;
        let manifest = TemplateManifest::from_str(toml_str).unwrap();
        assert!(manifest.template.dabs.is_none());
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
