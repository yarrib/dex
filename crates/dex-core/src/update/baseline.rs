//! Acquire the "old" and "new" rendered baselines an update diffs between.
//!
//! - **old baseline** (`A`): read from `.dex/cache/baseline/`, the tree that
//!   was rendered at the currently-recorded ref. Reading the cache — rather
//!   than re-rendering an old template that may no longer exist — is what makes
//!   updates work offline and for embedded templates whose old version is gone.
//! - **new template**: loaded from the recorded source so it can be rendered
//!   with the (possibly augmented) answers into the "new" baseline `B`.
//!
//! Remote ref resolution (latest-tag / worktree materialization) is layered on
//! top of this in a later step; here a remote source is loaded from its local
//! cache clone, and its ref is the clone's current HEAD.

use std::path::PathBuf;

use crate::config::{git_head_sha, remote_cache_dir};
use crate::error::{DexError, UpdateError};
use crate::scaffold::RenderedTree;
use crate::template::registry::load_template;
use crate::template::{Template, TemplateSource};
use crate::update::manifest::{StateManifest, load_baseline_cache};
use crate::update::{SourceKind, TemplateState};

/// The template at the update target, plus the ref it resolved to.
pub struct ResolvedTemplate {
    pub template: Template,
    pub new_ref: String,
}

/// Read the old rendered baseline (`A`) from the project's cache.
///
/// Errors with [`UpdateError::BaselineUnavailable`] if the cache is missing —
/// without it there's nothing to diff local edits against, and re-rendering the
/// old template revision is not always possible.
pub fn load_old_baseline(project_dir: &std::path::Path) -> Result<RenderedTree, DexError> {
    match load_baseline_cache(project_dir)? {
        Some(tree) => Ok(tree),
        None => Err(DexError::Update(UpdateError::BaselineUnavailable(
            "no rendered baseline in .dex/cache/baseline/".to_string(),
        ))),
    }
}

/// Load the template at the update target and resolve the ref it corresponds to.
///
/// `explicit_ref` is honored for remote sources (checkout that ref); for
/// embedded/directory sources the ref is always the template's own version.
pub fn resolve_new_template(
    manifest: &StateManifest,
    explicit_ref: Option<&str>,
) -> Result<ResolvedTemplate, DexError> {
    let state = &manifest.template;
    match state.source {
        SourceKind::Embedded => {
            let template = load_template(&TemplateSource::Embedded, &state.name)?;
            let new_ref = template.meta.version.clone();
            Ok(ResolvedTemplate { template, new_ref })
        }
        SourceKind::Directory => {
            let base = directory_base(state)?;
            let template = load_template(&TemplateSource::Directory(base), &state.name)?;
            let new_ref = template.meta.version.clone();
            Ok(ResolvedTemplate { template, new_ref })
        }
        SourceKind::Remote => resolve_remote_template(state, explicit_ref),
    }
}

fn directory_base(state: &TemplateState) -> Result<PathBuf, DexError> {
    let location = state.location.as_ref().ok_or_else(|| {
        DexError::Update(UpdateError::RefResolution {
            git_ref: state.git_ref.clone(),
            message: "directory template has no recorded location".to_string(),
        })
    })?;
    Ok(PathBuf::from(location))
}

/// Load a remote template from its local cache clone.
///
/// This resolves the clone's current HEAD as the new ref. Fetching/tag
/// selection is added by the remote-ref layer; here we render whatever the
/// cache currently points at, which is correct for already-synced caches.
fn resolve_remote_template(
    state: &TemplateState,
    _explicit_ref: Option<&str>,
) -> Result<ResolvedTemplate, DexError> {
    let cache = remote_cache_dir().join(&state.name);
    if !cache.is_dir() {
        return Err(DexError::Update(UpdateError::Offline(format!(
            "remote template cache '{}' is missing",
            cache.display()
        ))));
    }

    let template = load_template(&TemplateSource::Directory(cache.clone()), &state.name)?;
    let new_ref = git_head_sha(&cache).unwrap_or_else(|_| template.meta.version.clone());
    Ok(ResolvedTemplate { template, new_ref })
}
