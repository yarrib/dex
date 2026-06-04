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

    #[error(transparent)]
    Mcp(#[from] McpError),

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

/// Errors related to wiring the MCP server into client config files.
#[derive(Debug, thiserror::Error)]
pub enum McpError {
    #[error(
        "unknown MCP client: '{0}'. Valid clients: claude-code, claude-desktop, cursor, vscode, codex, zed, antigravity"
    )]
    UnknownClient(String),

    #[error("could not determine your home directory (needed for {0} config)")]
    HomeDirNotFound(&'static str),

    #[error("could not parse existing config at {path}: {message}")]
    Parse { path: String, message: String },

    #[error("config at {path} has a '{key}' entry that is not a table/object")]
    NotAnObject { path: String, key: String },

    #[error("failed to serialize config for {path}: {message}")]
    Serialize { path: String, message: String },
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
