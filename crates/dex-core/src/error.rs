//! Error types for dex-core.

use std::path::PathBuf;

/// Top-level error type for dex operations.
#[derive(Debug, thiserror::Error)]
pub enum DexError {
    #[error(transparent)]
    Config(#[from] ConfigError),

    #[error(transparent)]
    Template(#[from] TemplateError),

    #[error(transparent)]
    Skill(#[from] SkillError),

    #[error("render error: {0}")]
    Render(#[from] minijinja::Error),

    #[error("I/O error at {path}: {source}")]
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
}

/// Errors related to configuration parsing and validation.
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("config file not found: {0}")]
    NotFound(PathBuf),

    #[error("invalid config: {0}")]
    Invalid(String),

    #[error("config parse error: {0}")]
    Parse(#[from] toml::de::Error),
}

/// Errors related to skill pack operations.
#[derive(Debug, thiserror::Error)]
pub enum SkillError {
    #[error("skill pack not found: '{0}'")]
    PackNotFound(String),

    #[error("skills.toml parse error: {0}")]
    ManifestParse(String),

    #[error("unknown install target: '{0}'. Valid targets: claude, cursor, copilot, generic")]
    InvalidTarget(String),
}

/// Errors related to template operations.
#[derive(Debug, thiserror::Error)]
pub enum TemplateError {
    #[error("template not found: {0}")]
    NotFound(String),

    #[error("invalid template manifest: {0}")]
    InvalidManifest(String),

    #[error("missing required variable: {0}")]
    MissingVariable(String),

    #[error("variable validation failed for '{name}': {message}")]
    ValidationFailed { name: String, message: String },
}
