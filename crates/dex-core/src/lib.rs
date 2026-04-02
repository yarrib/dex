//! dex-core — template engine, config parsing, and scaffolding for dex.
//!
//! This crate contains all business logic. No UI, no terminal output, no Python
//! dependencies. It returns structured data; the calling layer renders it.

pub mod agent;
pub mod apply_trait;
pub mod config;
pub mod error;
pub mod scaffold;
pub mod skills;
pub mod template;
pub mod traits;

pub use agent::{AgentAnswers, AgentDeployTarget, AgentTrigger};
pub use apply_trait::{TraitResult, apply_trait};
pub use config::{
    DexConfig, PassthroughSpec, ProjectConfig, ProjectSkillsConfig, RemoteSource, load_dex_config,
    load_preset, load_project_config, load_standards, presets_path, record_trait, resolve_remote,
    resolve_skill_remote, skills_cache_dir,
};
pub use error::{DexError, SkillError};
pub use scaffold::{ScaffoldResult, scaffold};
pub use skills::{
    InstallResult, InstallTarget, SkillPack, SkillPackEntry, SkillSource, install_skills,
    list_packs, load_pack, load_pack_with_remote_fetch,
};
pub use template::{Template, TemplateMeta, TemplateSource};
pub use traits::{Trait, TraitMeta, list_traits, load_trait};

/// Result type alias for dex operations.
pub type Result<T> = std::result::Result<T, DexError>;
