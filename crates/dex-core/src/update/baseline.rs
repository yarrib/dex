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

use crate::config::{RemoteSource, remote_cache_dir, resolve_remote};
use crate::error::{DexError, UpdateError};
use crate::scaffold::RenderedTree;
use crate::template::registry::load_template;
use crate::template::{Template, TemplateSource};
use crate::update::manifest::{StateManifest, load_baseline_cache};
use crate::update::remote::{fetch_updates, load_template_at_ref, target_refish};
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

/// Resolve and load a remote template from its local cache clone.
///
/// Fetches updates (best-effort — offline is tolerated), selects the target ref
/// (explicit `--ref`, else latest tag, else the default branch head), and
/// materializes it via a detached worktree. The recorded ref is the resolved
/// commit SHA.
fn resolve_remote_template(
    state: &TemplateState,
    explicit_ref: Option<&str>,
) -> Result<ResolvedTemplate, DexError> {
    // The cache clone is keyed by the configured remote name, recorded at init.
    let remote_name = state.remote_name.clone().ok_or_else(|| {
        DexError::Update(UpdateError::RefResolution {
            git_ref: state.git_ref.clone(),
            message: "remote template state is missing its remote_name".to_string(),
        })
    })?;
    let cache = remote_cache_dir().join(&remote_name);

    // Re-clone if the cache was cleared (needs the URL from `location`).
    if !cache.is_dir()
        && let Some(url) = &state.location
    {
        let remote = RemoteSource {
            name: remote_name.clone(),
            url: url.clone(),
            git_ref: None,
        };
        resolve_remote(&remote, true)?;
    }
    if !cache.is_dir() {
        return Err(DexError::Update(UpdateError::Offline(format!(
            "remote template cache '{}' is missing and could not be cloned",
            cache.display()
        ))));
    }

    let online = fetch_updates(&cache);
    let refish = target_refish(&cache, explicit_ref);

    let revision = load_template_at_ref(&cache, &state.name, &refish).map_err(|e| {
        // Distinguish "we're offline and the ref isn't cached" from other errors.
        if !online {
            DexError::Update(UpdateError::Offline(format!(
                "could not resolve '{refish}' from the local cache while offline"
            )))
        } else {
            e
        }
    })?;

    Ok(ResolvedTemplate {
        template: revision.template,
        new_ref: revision.sha,
    })
}
