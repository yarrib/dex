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
pub use config::ProjectConfig;
pub use error::DexError;
pub use scaffold::{scaffold, ScaffoldResult};
pub use template::{Template, TemplateMeta, TemplateSource};

/// Result type alias for dex operations.
pub type Result<T> = std::result::Result<T, DexError>;
