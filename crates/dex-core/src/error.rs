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

    #[error(transparent)]
    Context(#[from] ContextError),

    #[error(transparent)]
    Update(#[from] UpdateError),

    #[error("render error: {0}")]
    Render(#[from] minijinja::Error),

    #[error("I/O error at {path}: {source}")]
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
}

/// Errors related to building the project-memory knowledge graph.
#[derive(Debug, thiserror::Error)]
pub enum ContextError {
    #[error("not a git repository (or git is unavailable): {0}")]
    NotARepo(String),

    #[error("git command failed: {0}")]
    Git(String),

    #[error("could not run git — is it installed and on PATH? ({0})")]
    GitSpawn(String),
}

/// Errors related to `dex update` (template re-application).
#[derive(Debug, thiserror::Error)]
pub enum UpdateError {
    #[error(
        "no update state found at {0} — this project predates `dex update`; re-run `dex init` with a current dex to record it"
    )]
    NoManifest(PathBuf),

    #[error("could not resolve ref '{git_ref}': {message}")]
    RefResolution { git_ref: String, message: String },

    #[error(
        "old baseline unavailable ({0}) — restore .dex/cache/ from a teammate or pass --ref to pick an explicit target"
    )]
    BaselineUnavailable(String),

    #[error("template source unreachable and ref not cached locally: {0}")]
    Offline(String),

    #[error("git command failed: {0}")]
    Git(String),

    #[error("already up to date (ref {0})")]
    AlreadyUpToDate(String),
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
