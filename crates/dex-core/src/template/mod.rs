//! Template system: engine, manifest parsing, registry, and variable handling.

pub mod engine;
pub mod manifest;
pub mod registry;
pub mod variables;

pub use engine::TemplateEngine;
pub use manifest::{FileRule, TemplateManifest};
pub use registry::TemplateSource;
pub use variables::{VariableSpec, VariableType};

use std::collections::HashMap;
use std::path::PathBuf;

/// Metadata about a template (for listing/selection).
#[derive(Debug, Clone)]
pub struct TemplateMeta {
    pub name: String,
    pub description: String,
    pub version: String,
    pub min_dex_version: Option<String>,
}

/// A fully-loaded template ready for rendering.
#[derive(Debug)]
pub struct Template {
    pub meta: TemplateMeta,
    pub variables: Vec<VariableSpec>,
    pub file_rules: Vec<FileRule>,
    /// Map from relative path to template content.
    pub files: HashMap<PathBuf, String>,
}
