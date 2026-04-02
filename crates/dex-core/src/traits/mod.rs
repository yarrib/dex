//! Dynamic traits: composable feature injection for dex projects.
//!
//! A trait is a mini-template that can be bolted onto an existing project.
//! Unlike `dex init` which scaffolds a project from scratch, `dex add <trait>`
//! injects a named, versioned component — files and/or patches to existing files.

pub mod manifest;
pub mod registry;

pub use manifest::{ConflictPolicy, PatchRule, TraitFileRule, TraitManifest, TraitMetaRaw};
pub use registry::{list_traits, load_trait};

use std::collections::HashMap;
use std::path::PathBuf;

/// Metadata about a trait (for listing/selection).
#[derive(Debug, Clone)]
pub struct TraitMeta {
    pub name: String,
    pub description: String,
    pub version: String,
    pub min_dex_version: Option<String>,
}

/// A fully-loaded trait ready to be applied.
#[derive(Debug)]
pub struct Trait {
    pub meta: TraitMeta,
    pub variables: Vec<crate::template::VariableSpec>,
    pub file_rules: Vec<TraitFileRule>,
    /// Map from relative path to file content.
    pub files: HashMap<PathBuf, String>,
    pub patches: Vec<PatchRule>,
}
