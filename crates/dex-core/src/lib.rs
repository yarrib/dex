//! dex-core — template engine, config parsing, and scaffolding for dex.
//!
//! This crate contains all business logic. No UI, no terminal output, no Python
//! dependencies. It returns structured data; the calling layer renders it.

pub mod agent;
pub mod config;
pub mod error;
pub mod scaffold;
pub mod template;

pub use agent::{AgentAnswers, AgentDeployTarget, AgentTrigger};
pub use config::{
    DexConfig, PassthroughSpec, ProjectConfig, RemoteSource, load_dex_config, load_preset,
    load_project_config, load_standards, presets_path, resolve_remote,
};
pub use error::DexError;
pub use scaffold::{ScaffoldResult, scaffold};
pub use template::{Template, TemplateMeta, TemplateSource};

/// Result type alias for dex operations.
pub type Result<T> = std::result::Result<T, DexError>;
