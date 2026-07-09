//! `dex update` — re-apply template changes to an already-generated project.
//!
//! State lives in a project-local `.dex/` directory (analogous to `.git/`):
//!
//! ```text
//! .dex/
//! ├── manifest.toml      # template source, ref, answers — update-critical state
//! ├── history.toml       # append-only log of past updates
//! ├── .gitignore         # ignores cache/
//! └── cache/baseline/    # rendered tree at the recorded ref (offline baseline)
//! ```

pub mod manifest;
pub mod merge;
pub mod report;

pub use merge::{FileAction, FileChange, merge_trees};
pub use report::UpdateReport;

pub use manifest::{
    HistoryEntry, SCHEMA_VERSION, SourceKind, StateManifest, TemplateState, UpdateHooks,
    append_history, build_state_manifest, load_baseline_cache, load_history, load_state_manifest,
    record_project_state, save_state_manifest, typed_answers, write_baseline_cache,
    write_project_state,
};
