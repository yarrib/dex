//! Parsing of `databricks_template_schema.json` for unified prompt mode.
//!
//! DABs templates declare their variables in a JSON Schema-like file at the
//! template root. In unified mode, dex reads this schema, converts properties
//! to dex VariableSpecs, and presents a single prompt flow.

use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

use crate::error::DexError;
use crate::template::variables::{VariableSpec, VariableType};

/// Top-level structure of `databricks_template_schema.json`.
#[derive(Debug, Deserialize)]
pub struct DabsTemplateSchema {
    #[serde(default)]
    pub welcome_message: Option<String>,
    #[serde(default)]
    pub success_message: Option<String>,
    #[serde(default)]
    pub min_databricks_cli_version: Option<String>,
    #[serde(default)]
    pub properties: HashMap<String, DabsSchemaProperty>,
}

/// A single property (variable) in the DABs template schema.
#[derive(Debug, Deserialize)]
pub struct DabsSchemaProperty {
    #[serde(rename = "type", default = "default_string")]
    pub prop_type: String,
    #[serde(default)]
    pub default: Option<serde_json::Value>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub order: Option<i32>,
    #[serde(rename = "enum", default)]
    pub enum_values: Option<Vec<String>>,
    #[serde(default)]
    pub pattern: Option<String>,
    #[serde(default)]
    pub skip_prompt_if: Option<serde_json::Value>,
}

fn default_string() -> String {
    "string".to_string()
}

impl DabsTemplateSchema {
    /// Parse a `databricks_template_schema.json` from a file path.
    pub fn from_path(path: &Path) -> Result<Self, DexError> {
        let content = std::fs::read_to_string(path).map_err(|source| DexError::Io {
            path: path.to_path_buf(),
            source,
        })?;
        Self::parse(&content)
    }

    /// Parse from a JSON string.
    pub fn parse(content: &str) -> Result<Self, DexError> {
        serde_json::from_str(content).map_err(|e| {
            DexError::Template(crate::error::TemplateError::InvalidManifest(format!(
                "invalid databricks_template_schema.json: {e}"
            )))
        })
    }

    /// Convert DABs schema properties to dex VariableSpecs, sorted by `order`.
    #[must_use]
    pub fn to_variable_specs(&self) -> Vec<VariableSpec> {
        let mut props: Vec<_> = self.properties.iter().collect();
        props.sort_by_key(|(_, p)| p.order.unwrap_or(999));

        props
            .into_iter()
            .map(|(name, prop)| {
                let (var_type, choices) = if let Some(ref enums) = prop.enum_values {
                    (VariableType::Choice, Some(enums.clone()))
                } else if prop.prop_type == "boolean" {
                    (VariableType::Bool, None)
                } else {
                    (VariableType::String, None)
                };

                let default = prop.default.as_ref().map(|d| match d {
                    serde_json::Value::String(s) => toml::Value::String(s.clone()),
                    serde_json::Value::Bool(b) => toml::Value::Boolean(*b),
                    serde_json::Value::Number(n) => {
                        if let Some(i) = n.as_i64() {
                            toml::Value::Integer(i)
                        } else {
                            toml::Value::String(n.to_string())
                        }
                    }
                    other => toml::Value::String(other.to_string()),
                });

                VariableSpec {
                    name: name.clone(),
                    prompt: prop.description.clone().unwrap_or_else(|| name.clone()),
                    var_type,
                    default,
                    required: prop.default.is_none(),
                    choices,
                    validate: prop.pattern.clone(),
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_dabs_schema() {
        let json = r#"{
            "welcome_message": "Welcome!",
            "properties": {
                "project_name": {
                    "type": "string",
                    "default": "my_project",
                    "description": "Unique name for this project",
                    "order": 1,
                    "pattern": "^[a-zA-Z0-9_]+$"
                },
                "include_notebook": {
                    "type": "string",
                    "default": "yes",
                    "description": "Include a notebook?",
                    "order": 2,
                    "enum": ["yes", "no"]
                }
            },
            "success_message": "Created {{.project_name}}"
        }"#;

        let schema = DabsTemplateSchema::parse(json).unwrap();
        assert_eq!(schema.welcome_message.as_deref(), Some("Welcome!"));
        assert_eq!(schema.properties.len(), 2);

        let vars = schema.to_variable_specs();
        assert_eq!(vars.len(), 2);
        // Sorted by order
        assert_eq!(vars[0].name, "project_name");
        assert_eq!(vars[0].validate.as_deref(), Some("^[a-zA-Z0-9_]+$"));
        assert_eq!(vars[1].name, "include_notebook");
        assert!(vars[1].choices.is_some());
    }

    #[test]
    fn parse_minimal_schema() {
        let json = r#"{"properties": {}}"#;
        let schema = DabsTemplateSchema::parse(json).unwrap();
        assert!(schema.properties.is_empty());
        assert!(schema.to_variable_specs().is_empty());
    }
}
