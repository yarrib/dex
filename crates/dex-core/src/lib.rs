//! dex-core — template engine, config parsing, and scaffolding for dex.
//!
//! This crate contains all business logic. No UI, no terminal output, no Python
//! dependencies. It returns structured data; the calling layer renders it.

pub mod apply_trait;
pub mod config;
pub mod context_graph;
pub mod context_map;
pub mod error;
pub mod mcp;
pub mod scaffold;
pub mod skills;
pub mod template;
pub mod traits;

pub use apply_trait::{TraitResult, apply_trait};
pub use config::{
    DexConfig, PassthroughSpec, ProjectConfig, ProjectSkillsConfig, RemoteSource, load_answers,
    load_dex_config, load_preset, load_project_config, load_standards, presets_path, record_trait,
    resolve_remote, resolve_skill_remote, save_answers, skills_cache_dir,
};
pub use context_graph::{
    Edge, EdgeKind, FunctionalArea, Node, NodeClass, SyncOptions, SyncReport,
    sync as sync_context_graph,
};
pub use context_map::{ContextMap, write_context_map};
pub use error::{ContextError, DexError, McpError, SkillError};
pub use mcp::{McpClient, McpInstallPlan, apply_mcp_plan, build_client_config, plan_mcp_client};
pub use scaffold::{ScaffoldResult, scaffold};
pub use skills::{
    InstallResult, InstallTarget, SkillPack, SkillPackEntry, SkillSource, install_skills,
    list_packs, load_pack, load_pack_with_remote_fetch,
};
pub use template::OnSuccessSpec;
pub use template::{Template, TemplateMeta, TemplateSource};
pub use traits::{Trait, TraitMeta, list_traits, load_trait};

/// Result type alias for dex operations.
pub type Result<T> = std::result::Result<T, DexError>;
