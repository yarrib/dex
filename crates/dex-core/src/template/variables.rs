//! Template variable specifications and validation.

use serde::Deserialize;

use crate::error::TemplateError;

/// A variable declaration (resolved, with name) used throughout the system.
#[derive(Debug, Clone, Deserialize)]
pub struct VariableSpec {
    pub name: String,
    pub prompt: String,
    #[serde(rename = "type", default = "default_var_type")]
    pub var_type: VariableType,
    #[serde(default)]
    pub default: Option<toml::Value>,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub choices: Option<Vec<String>>,
    #[serde(default)]
    pub validate: Option<String>,
    /// Optional prompt ordering. Variables with `order` are presented first,
    /// sorted by this value. Variables without `order` follow in their
    /// definition order.
    #[serde(default)]
    pub order: Option<u32>,
}

/// Inline variable spec used when parsing the `[variables]` table format.
/// The variable name comes from the map key, not a field.
#[derive(Debug, Clone, Deserialize)]
pub struct VariableSpecInline {
    pub prompt: String,
    #[serde(rename = "type", default = "default_var_type")]
    pub var_type: VariableType,
    #[serde(default)]
    pub default: Option<toml::Value>,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub choices: Option<Vec<String>>,
    #[serde(default)]
    pub validate: Option<String>,
    #[serde(default)]
    pub order: Option<u32>,
}

impl VariableSpecInline {
    pub fn into_spec(self, name: String) -> VariableSpec {
        VariableSpec {
            name,
            prompt: self.prompt,
            var_type: self.var_type,
            default: self.default,
            required: self.required,
            choices: self.choices,
            validate: self.validate,
            order: self.order,
        }
    }
}

/// The type of a template variable.
#[derive(Debug, Clone, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum VariableType {
    #[default]
    String,
    Bool,
    Choice,
    Multi,
}

fn default_var_type() -> VariableType {
    VariableType::String
}

impl VariableSpec {
    /// Validate a string value against this variable's constraints.
    pub fn validate_value(&self, value: &str) -> Result<(), TemplateError> {
        // Check against regex pattern if present.
        if let Some(pattern) = &self.validate {
            let re = regex::Regex::new(pattern).map_err(|e| TemplateError::ValidationFailed {
                name: self.name.clone(),
                message: format!("invalid validation pattern: {e}"),
            })?;
            if !re.is_match(value) {
                return Err(TemplateError::ValidationFailed {
                    name: self.name.clone(),
                    message: format!("value '{value}' does not match pattern '{pattern}'"),
                });
            }
        }

        // Check against choices if present.
        if let Some(choices) = &self.choices
            && !choices.iter().any(|c| c == value)
        {
            return Err(TemplateError::ValidationFailed {
                name: self.name.clone(),
                message: format!("value '{value}' is not one of: {}", choices.join(", ")),
            });
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_spec(validate: Option<&str>, choices: Option<Vec<&str>>) -> VariableSpec {
        VariableSpec {
            name: "test".into(),
            prompt: "Test".into(),
            var_type: VariableType::String,
            default: None,
            required: false,
            choices: choices.map(|c| c.into_iter().map(String::from).collect()),
            validate: validate.map(String::from),
            order: None,
        }
    }

    #[test]
    fn validate_regex_pass() {
        let spec = make_spec(Some("^[a-z]+$"), None);
        assert!(spec.validate_value("hello").is_ok());
    }

    #[test]
    fn validate_regex_fail() {
        let spec = make_spec(Some("^[a-z]+$"), None);
        assert!(spec.validate_value("Hello123").is_err());
    }

    #[test]
    fn validate_choices_pass() {
        let spec = make_spec(None, Some(vec!["a", "b", "c"]));
        assert!(spec.validate_value("b").is_ok());
    }

    #[test]
    fn validate_choices_fail() {
        let spec = make_spec(None, Some(vec!["a", "b", "c"]));
        assert!(spec.validate_value("d").is_err());
    }
}
