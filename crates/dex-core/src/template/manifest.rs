//! Template manifest (`template.toml`) parsing.

use indexmap::IndexMap;
use serde::Deserialize;
use std::path::Path;

use crate::error::{DexError, TemplateError};
use crate::template::variables::VariableSpecInline;
use crate::template::{TemplateMeta, VariableSpec};

/// Raw deserialized template manifest from `template.toml`.
#[derive(Debug, Deserialize)]
pub struct TemplateManifest {
    pub template: TemplateMetaRaw,
    #[serde(default)]
    variables: VariablesField,
    #[serde(default)]
    pub files: Vec<FileRule>,
    #[serde(default)]
    pub hooks: Option<HooksSpec>,
}

/// Supports both `[[variables]]` array format and `[variables]` inline table format.
///
/// - `[[variables]]` — array of tables, each with an explicit `name` field (legacy)
/// - `[variables]` — inline table where the key is the variable name (preferred)
#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum VariablesField {
    /// `[[variables]]` array format — name is an explicit field.
    Array(Vec<VariableSpec>),
    /// `[variables]` inline format — name comes from the map key.
    Map(IndexMap<String, VariableSpecInline>),
}

impl Default for VariablesField {
    fn default() -> Self {
        VariablesField::Array(vec![])
    }
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

    /// How DABs variables are collected. One of:
    /// - `"passthrough"` (default) — let `databricks bundle init` prompt interactively
    /// - `"unified"` — dex reads databricks_template_schema.json, merges prompts
    /// - `"mapped"` — pre-fill via variable_map, DABs prompts for unmapped vars
    #[serde(default)]
    pub prompt: DabsPromptMode,

    /// Maps dex variable names to DABs template variable names.
    /// Used in "mapped" mode to write a config JSON for partial pre-fill.
    #[serde(default)]
    pub variable_map: std::collections::HashMap<String, String>,

    /// Overrides for DABs schema variables (unified mode).
    /// Keys are DABs variable names. Values override default, choices, etc.
    #[serde(default)]
    pub overrides: std::collections::HashMap<String, DabsVariableOverride>,
}

/// How DABs template variables are prompted during `dex init`.
#[derive(Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DabsPromptMode {
    /// Let `databricks bundle init` handle prompts interactively.
    #[default]
    Passthrough,
    /// dex reads the DABs schema, merges with dex variables, single prompt flow.
    Unified,
    /// Pre-fill mapped variables, DABs prompts for the rest.
    Mapped,
}

/// Override for a specific DABs schema variable (unified mode).
#[derive(Debug, Deserialize)]
pub struct DabsVariableOverride {
    #[serde(default)]
    pub default: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub choices: Option<Vec<String>>,
    /// If true, skip this variable (don't prompt, use default).
    #[serde(default)]
    pub skip: bool,
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
        Self::parse(&content)
    }

    /// Parse a `template.toml` from a string.
    pub fn parse(content: &str) -> Result<Self, DexError> {
        toml::from_str(content)
            .map_err(|e| DexError::Template(TemplateError::InvalidManifest(e.to_string())))
    }

    /// Return variables in prompt order.
    ///
    /// Variables with an explicit `order` field are sorted by that value first.
    /// Variables without `order` follow in their original definition order.
    /// This means you can mix ordered and unordered variables: ordered ones are
    /// promoted to the front, unordered ones trail in document order.
    #[must_use]
    pub fn variables(&self) -> Vec<VariableSpec> {
        let mut vars: Vec<VariableSpec> = match &self.variables {
            VariablesField::Array(v) => v.clone(),
            VariablesField::Map(m) => m
                .iter()
                .map(|(name, spec)| spec.clone().into_spec(name.clone()))
                .collect(),
        };
        // Stable sort: (has_no_order=false, order) < (has_no_order=true, 0)
        // so explicitly-ordered vars come first, unordered trail in original position.
        vars.sort_by_key(|v| (v.order.is_none(), v.order.unwrap_or(0)));
        vars
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
        let manifest = TemplateManifest::parse(toml_str).unwrap();
        assert_eq!(manifest.template.name, "default");
        assert!(manifest.variables().is_empty());
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
        let manifest = TemplateManifest::parse(toml_str).unwrap();
        assert_eq!(manifest.variables().len(), 2);
        let dabs = manifest.template.dabs.unwrap();
        assert!(dabs.source.contains("bundle-examples"));
        assert_eq!(
            dabs.variable_map.get("project_name").unwrap(),
            "project_name"
        );
    }

    #[test]
    fn parse_standalone_manifest_has_no_dabs() {
        let toml_str = r#"
            [template]
            name = "default"
            description = "Standalone template"
            version = "0.1.0"
        "#;
        let manifest = TemplateManifest::parse(toml_str).unwrap();
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
        let manifest = TemplateManifest::parse(toml_str).unwrap();
        assert_eq!(manifest.variables().len(), 2);
        assert_eq!(manifest.files.len(), 1);
        assert!(manifest.hooks.unwrap().post_scaffold.is_some());
    }

    #[test]
    fn parse_inline_variables_format() {
        let toml_str = r#"
            [template]
            name = "default"
            description = "Default project template"
            version = "0.1.0"

            [variables]
            project_name = { prompt = "Project name", required = true, order = 1 }
            python_version = { prompt = "Python version", type = "choice", choices = ["3.12", "3.11"], default = "3.12", order = 2 }
            author = { prompt = "Author name" }
        "#;
        let manifest = TemplateManifest::parse(toml_str).unwrap();
        let vars = manifest.variables();
        assert_eq!(vars.len(), 3);
        // ordered vars come first
        assert_eq!(vars[0].name, "project_name");
        assert_eq!(vars[1].name, "python_version");
        // unordered trails
        assert_eq!(vars[2].name, "author");
    }

    #[test]
    fn inline_variables_order_field_sorts_correctly() {
        let toml_str = r#"
            [template]
            name = "test"
            description = "Test"
            version = "0.1.0"

            [variables]
            b_var = { prompt = "B", order = 2 }
            a_var = { prompt = "A", order = 1 }
            c_var = { prompt = "C" }
        "#;
        let manifest = TemplateManifest::parse(toml_str).unwrap();
        let vars = manifest.variables();
        assert_eq!(vars[0].name, "a_var");
        assert_eq!(vars[1].name, "b_var");
        assert_eq!(vars[2].name, "c_var");
    }
}
