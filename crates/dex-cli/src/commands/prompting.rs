//! Shared interactive variable prompting for `dex init` and `dex update`.
//!
//! Both commands resolve a template's variables into a `minijinja` context the
//! same way: honor `when` conditions, take pre-filled/known answers verbatim,
//! and otherwise prompt (or fall back to defaults under `--no-prompt`). This
//! module owns the per-variable prompt so the two commands can't drift.

use std::collections::HashMap;

use dialoguer::{Confirm, Input, MultiSelect, Select};

use dex_core::DexError;
use dex_core::template::variables::{VariableSpec, VariableType};

/// Produce a value for one variable, either by prompting or — under
/// `no_prompt` — from its effective default.
///
/// `effective_default` is the already-resolved default as a string (the caller
/// applies any special-casing, e.g. deriving `project_name` from the dir).
pub fn prompt_variable(
    spec: &VariableSpec,
    effective_default: &str,
    no_prompt: bool,
) -> Result<minijinja::Value, DexError> {
    if no_prompt {
        let val = match spec.var_type {
            VariableType::Bool => {
                let b = effective_default.is_empty() || effective_default == "true";
                minijinja::Value::from(b)
            }
            VariableType::Choice => {
                let v = if effective_default.is_empty() {
                    spec.choices
                        .as_ref()
                        .and_then(|c| c.first().cloned())
                        .unwrap_or_default()
                } else {
                    effective_default.to_string()
                };
                minijinja::Value::from(v)
            }
            _ => minijinja::Value::from(effective_default.to_string()),
        };
        return Ok(val);
    }

    let val = match spec.var_type {
        VariableType::Choice => {
            let choices = spec.choices.as_deref().unwrap_or(&[]);
            let default_idx = choices
                .iter()
                .position(|c| c == effective_default)
                .unwrap_or(0);
            let selection = Select::new()
                .with_prompt(&spec.prompt)
                .items(choices)
                .default(default_idx)
                .interact()
                .map_err(io_error)?;
            minijinja::Value::from(choices[selection].clone())
        }
        VariableType::Multi => {
            let choices = spec.choices.as_deref().unwrap_or(&[]);
            let preselected: Vec<&str> = effective_default
                .split(',')
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .collect();
            let defaults: Vec<bool> = choices
                .iter()
                .map(|c| preselected.contains(&c.as_str()))
                .collect();
            let selections = MultiSelect::new()
                .with_prompt(&spec.prompt)
                .items(choices)
                .defaults(&defaults)
                .interact()
                .map_err(io_error)?;
            let picked: Vec<String> = selections.into_iter().map(|i| choices[i].clone()).collect();
            minijinja::Value::from(picked.join(","))
        }
        VariableType::Bool => {
            let default = effective_default.is_empty() || effective_default == "true";
            let answer = Confirm::new()
                .with_prompt(&spec.prompt)
                .default(default)
                .interact()
                .map_err(io_error)?;
            minijinja::Value::from(answer)
        }
        _ => {
            let mut input = Input::<String>::new().with_prompt(&spec.prompt);
            if !effective_default.is_empty() {
                input = input.default(effective_default.to_string());
            }

            if let Some(pattern) = &spec.validate {
                let re = regex::Regex::new(pattern).ok();
                let pattern_str = pattern.clone();
                input = input.validate_with(move |val: &String| -> Result<(), String> {
                    if let Some(ref re) = re {
                        if re.is_match(val) {
                            Ok(())
                        } else {
                            Err(format!(
                                "value '{val}' does not match pattern '{pattern_str}'"
                            ))
                        }
                    } else {
                        Ok(())
                    }
                });
            }

            let answer = input.interact_text().map_err(io_error)?;
            minijinja::Value::from(answer)
        }
    };
    Ok(val)
}

/// Convert a typed TOML value (from a saved answer) into a minijinja value,
/// preserving bool → bool and string → string so template conditionals behave
/// correctly on replay.
#[must_use]
pub fn toml_val_to_minijinja(v: &toml::Value) -> minijinja::Value {
    match v {
        toml::Value::Boolean(b) => minijinja::Value::from(*b),
        toml::Value::String(s) => minijinja::Value::from(s.clone()),
        toml::Value::Integer(i) => minijinja::Value::from(*i),
        toml::Value::Float(f) => minijinja::Value::from(*f),
        _ => minijinja::Value::from(v.to_string()),
    }
}

/// Render a TOML default value as a display string for prompting.
#[must_use]
pub fn toml_value_to_string(v: &toml::Value) -> String {
    match v {
        toml::Value::String(s) => s.clone(),
        toml::Value::Boolean(b) => b.to_string(),
        toml::Value::Integer(i) => i.to_string(),
        toml::Value::Float(f) => f.to_string(),
        other => other.to_string(),
    }
}

/// Evaluate a Jinja2 boolean expression against already-resolved variables.
///
/// Returns `true` for a truthy result, `false` otherwise (including on
/// evaluation errors, so a bad `when` expression skips the variable rather
/// than crashing the prompt loop).
#[must_use]
pub fn evaluate_when(expr: &str, vars: &HashMap<String, minijinja::Value>) -> bool {
    let env = minijinja::Environment::new();
    let source = format!("{{% if {expr} %}}true{{% else %}}false{{% endif %}}");
    env.render_str(&source, vars)
        .is_ok_and(|r| r.trim() == "true")
}

/// The default value a skipped (`when` false) variable takes, typed by its
/// declared kind.
#[must_use]
pub fn skipped_default(spec: &VariableSpec, effective_default: &str) -> minijinja::Value {
    match spec.var_type {
        VariableType::Bool => {
            let b = effective_default.is_empty() || effective_default == "true";
            minijinja::Value::from(b)
        }
        _ => minijinja::Value::from(effective_default.to_string()),
    }
}

pub(crate) fn io_error(e: impl std::fmt::Display) -> DexError {
    DexError::Io {
        path: std::path::PathBuf::from("<stdin>"),
        source: std::io::Error::other(e.to_string()),
    }
}
