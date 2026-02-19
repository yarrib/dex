//! Project configuration (`dex.toml`) parsing and types.

use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

use crate::error::{ConfigError, DexError};

/// Top-level project configuration from `dex.toml`.
#[derive(Debug, Deserialize)]
pub struct ProjectConfig {
    pub project: ProjectMeta,

    #[serde(default)]
    pub tasks: HashMap<String, TaskSpec>,

    #[serde(default)]
    pub profiles: HashMap<String, ProfileSpec>,

    #[serde(default)]
    pub passthrough: HashMap<String, PassthroughSpec>,
}

/// Project metadata from `[project]` section.
#[derive(Debug, Deserialize)]
pub struct ProjectMeta {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub template: Option<String>,
}

/// Task definition from `[tasks.*]` section.
#[derive(Debug, Deserialize)]
pub struct TaskSpec {
    pub command: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub depends_on: Vec<String>,
}

/// Environment profile from `[profiles.*]` section.
#[derive(Debug, Deserialize)]
pub struct ProfileSpec {
    #[serde(default)]
    pub workspace_url: Option<String>,
    #[serde(default)]
    pub cluster_id: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, toml::Value>,
}

/// Pass-through command spec from `[passthrough.*]` section.
#[derive(Debug, Deserialize)]
pub struct PassthroughSpec {
    pub command: String,
    #[serde(default)]
    pub description: Option<String>,
}

/// Load and parse a `dex.toml` project config from a file path.
pub fn load_project_config(path: &Path) -> Result<ProjectConfig, DexError> {
    let content = std::fs::read_to_string(path).map_err(|source| {
        if source.kind() == std::io::ErrorKind::NotFound {
            DexError::Config(ConfigError::NotFound(path.to_path_buf()))
        } else {
            DexError::Io {
                path: path.to_path_buf(),
                source,
            }
        }
    })?;

    let config: ProjectConfig = toml::from_str(&content).map_err(ConfigError::Parse)?;
    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_minimal_config() {
        let toml_str = r#"
            [project]
            name = "my-project"
        "#;
        let config: ProjectConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.project.name, "my-project");
        assert!(config.tasks.is_empty());
        assert!(config.profiles.is_empty());
    }

    #[test]
    fn parse_full_config() {
        let toml_str = r#"
            [project]
            name = "ml-pipeline"
            description = "Revenue forecasting"
            template = "ml-pipeline"

            [tasks.test]
            command = "pytest tests/"
            description = "Run tests"

            [tasks.build]
            command = "python -m build"
            depends_on = ["test"]

            [profiles.dev]
            workspace_url = "https://dev.cloud.databricks.com"
            cluster_id = "0123-456789-abcdef"

            [passthrough.db]
            command = "databricks"
            description = "Databricks CLI"
        "#;
        let config: ProjectConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.project.name, "ml-pipeline");
        assert_eq!(config.tasks.len(), 2);
        assert_eq!(config.tasks["build"].depends_on, vec!["test"]);
        assert_eq!(config.profiles["dev"].cluster_id.as_deref(), Some("0123-456789-abcdef"));
        assert_eq!(config.passthrough["db"].command, "databricks");
    }
}
