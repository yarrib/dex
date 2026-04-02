//! Skill pack management — discovery, loading, and installation.
//!
//! Skill packs are named collections of AI agent skills (Claude Code slash
//! commands and agent personas) defined as plain markdown files with a
//! `skills.toml` manifest.
//!
//! Packs can come from:
//! - **Embedded** — built-in packs compiled into the binary
//! - **Local directory** — org-managed path on disk
//! - **Remote git repository** — pulled to `~/.cache/dex/skills/`

pub mod installer;
pub mod manifest;
pub mod registry;

pub use installer::{InstallResult, InstallTarget, install_skills};
pub use manifest::{PackMeta, SkillPackManifest, SkillSpec, SkillType};
pub use registry::{
    SkillPack, SkillPackEntry, SkillSource, list_packs, load_pack, load_pack_with_remote_fetch,
};
