//! Plan and apply a `dex update`: render the new baseline, merge it into the
//! working tree, and (on apply) write files, refresh state, and log history.
//!
//! Split into plan → apply so the CLI can preview with `--dry-run` before
//! anything touches disk (mirrors `plan_mcp_client` / `apply_mcp_plan`).

use std::collections::{BTreeMap, HashMap};
use std::path::Path;

use crate::error::DexError;
use crate::scaffold::{RenderedTree, render_tree};
use crate::update::baseline::{ResolvedTemplate, load_old_baseline};
use crate::update::manifest::{
    StateManifest, TemplateState, UpdateHooks, append_history, history_entry_today,
    save_state_manifest, typed_answers, write_baseline_cache,
};
use crate::update::merge::{FileAction, FileChange, merge_trees};
use crate::update::report::UpdateReport;

/// A computed update, ready to preview or apply.
pub struct UpdatePlan {
    pub report: UpdateReport,
    pub changes: Vec<FileChange>,
    pub new_ref: String,
    pub new_version: Option<String>,
    /// The new render (`B`) — becomes the refreshed baseline cache on apply.
    pub new_tree: RenderedTree,
    /// Answers to persist (existing + any newly-answered variables).
    pub new_answers: BTreeMap<String, toml::Value>,
    /// Hooks carried forward from the new template.
    pub new_hooks: UpdateHooks,
}

impl UpdatePlan {
    /// True when applying would change nothing (no writes, deletes, conflicts).
    #[must_use]
    pub fn is_noop(&self) -> bool {
        self.report.is_empty()
    }
}

/// Compute the update without touching the filesystem.
pub fn plan_update(
    project_dir: &Path,
    manifest: &StateManifest,
    resolved: &ResolvedTemplate,
    variables: &HashMap<String, minijinja::Value>,
) -> Result<UpdatePlan, DexError> {
    let old = load_old_baseline(project_dir)?;
    let new_tree = render_tree(&resolved.template, variables)?;

    // Base = A (old render), ours = working tree, theirs = B (new render).
    // SEAM: --smart — the conflict hunks in `changes`/`report.conflicts` are
    // exactly the inputs a future LLM-assisted reconciliation would consume.
    let changes = merge_trees(&old, &new_tree, project_dir)?;
    let report =
        UpdateReport::from_changes(&manifest.template.git_ref, &resolved.new_ref, &changes);

    let new_answers = typed_answers(&resolved.template.variables, variables);
    let new_hooks = resolved
        .template
        .hooks
        .as_ref()
        .map(UpdateHooks::from)
        .unwrap_or_default();

    Ok(UpdatePlan {
        report,
        changes,
        new_ref: resolved.new_ref.clone(),
        new_version: Some(resolved.template.meta.version.clone()),
        new_tree,
        new_answers,
        new_hooks,
    })
}

/// Apply a previously computed plan: write/delete files, refresh the baseline
/// cache and manifest, and append a history entry.
pub fn apply_update(
    project_dir: &Path,
    manifest: &StateManifest,
    plan: &UpdatePlan,
    dex_version: &str,
) -> Result<(), DexError> {
    for change in &plan.changes {
        apply_change(project_dir, change)?;
    }

    // Next update diffs against this render, so the cache tracks B.
    write_baseline_cache(project_dir, &plan.new_tree)?;

    let new_manifest = StateManifest {
        schema_version: manifest.schema_version,
        template: TemplateState {
            name: manifest.template.name.clone(),
            source: manifest.template.source,
            location: manifest.template.location.clone(),
            remote_name: manifest.template.remote_name.clone(),
            git_ref: plan.new_ref.clone(),
            version: plan.new_version.clone(),
            dex_version: Some(dex_version.to_string()),
        },
        answers: plan.new_answers.clone(),
        hooks: plan.new_hooks.clone(),
    };
    save_state_manifest(project_dir, &new_manifest)?;

    append_history(
        project_dir,
        &history_entry_today(
            &manifest.template.git_ref,
            &plan.new_ref,
            Some(dex_version),
            plan.report.files_changed(),
            plan.report.conflicts.len(),
        ),
    )?;

    Ok(())
}

fn apply_change(project_dir: &Path, change: &FileChange) -> Result<(), DexError> {
    let dest = project_dir.join(&change.path);
    match change.action {
        FileAction::Deleted => {
            if dest.exists() {
                std::fs::remove_file(&dest).map_err(|source| DexError::Io {
                    path: dest.clone(),
                    source,
                })?;
            }
            Ok(())
        }
        // Unchanged / DeleteConflict / LocallyDeleted / BinaryConflict carry no
        // content and leave the working tree untouched.
        _ => {
            let Some(content) = &change.content else {
                return Ok(());
            };
            if let Some(parent) = dest.parent()
                && !parent.exists()
            {
                std::fs::create_dir_all(parent).map_err(|source| DexError::Io {
                    path: parent.to_path_buf(),
                    source,
                })?;
            }
            std::fs::write(&dest, content).map_err(|source| DexError::Io {
                path: dest.clone(),
                source,
            })
        }
    }
}
