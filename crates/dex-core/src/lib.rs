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
pub mod update;

pub use apply_trait::{TraitResult, apply_trait};
pub use config::{
    DexConfig, PassthroughSpec, ProjectConfig, ProjectSkillsConfig, RemoteSource, git_head_sha,
    load_answers, load_dex_config, load_preset, load_project_config, load_standards, presets_path,
    record_trait, remote_cache_dir, resolve_remote, resolve_skill_remote, save_answers,
    skills_cache_dir,
};
pub use context_graph::{
    Edge, EdgeKind, ExportOptions, ExportReport, FunctionalArea, Node, NodeClass, SyncOptions,
    SyncReport, export as export_context_graph, sync as sync_context_graph,
};
pub use context_map::{ContextMap, write_context_map};
pub use error::{ContextError, DexError, McpError, SkillError, UpdateError};
pub use mcp::{McpClient, McpInstallPlan, apply_mcp_plan, build_client_config, plan_mcp_client};
pub use scaffold::{RenderedTree, ScaffoldResult, render_tree, scaffold, write_tree};
pub use skills::{
    InstallResult, InstallTarget, SkillPack, SkillPackEntry, SkillSource, install_skills,
    list_packs, load_pack, load_pack_with_remote_fetch,
};
pub use template::OnSuccessSpec;
pub use template::{HooksSpec, Template, TemplateMeta, TemplateSource};
pub use traits::{Trait, TraitMeta, list_traits, load_trait};
pub use update::{
    FileAction, FileChange, HistoryEntry, ResolvedTemplate, SourceKind, StateManifest,
    TemplateState, UpdateHooks, UpdatePlan, UpdateReport, apply_update, load_state_manifest,
    merge_trees, plan_update, record_project_state, resolve_new_template, save_state_manifest,
    typed_answers, write_project_state,
};

/// Result type alias for dex operations.
pub type Result<T> = std::result::Result<T, DexError>;
