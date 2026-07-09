//! Remote template ref resolution for `dex update`.
//!
//! Given the local cache clone of a remote template repo (created by
//! `resolve_remote` at init/discovery time), this fetches updates, picks the
//! target ref (explicit, else latest semver tag, else the remote default
//! branch), and materializes that revision via a detached `git worktree` so the
//! template can be loaded and rendered. The worktree is always cleaned up.

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::error::{DexError, UpdateError};
use crate::template::registry::load_template;
use crate::template::{Template, TemplateSource};

/// A resolved remote revision: the loaded template plus the commit SHA it was
/// materialized from.
pub struct RemoteRevision {
    pub template: Template,
    pub sha: String,
}

/// Fetch updates for the cache clone. Best-effort — failure (e.g. offline) is
/// reported so the caller can fall back to whatever is cached locally.
#[must_use]
pub fn fetch_updates(repo: &Path) -> bool {
    git(repo, &["fetch", "--tags", "--force", "origin"])
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Latest tag by descending version sort, if the repo has any tags.
#[must_use]
pub fn latest_tag(repo: &Path) -> Option<String> {
    let out = git(repo, &["tag", "--sort=-v:refname"]).ok()?;
    if !out.status.success() {
        return None;
    }
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .map(str::trim)
        .find(|l| !l.is_empty())
        .map(str::to_string)
}

/// Resolve a ref (tag, branch, or SHA) to a commit SHA within the repo.
pub fn resolve_ref_sha(repo: &Path, refish: &str) -> Result<String, DexError> {
    let spec = format!("{refish}^{{commit}}");
    let out = git(repo, &["rev-parse", "--verify", "--quiet", &spec]).map_err(|e| {
        DexError::Update(UpdateError::RefResolution {
            git_ref: refish.to_string(),
            message: e.to_string(),
        })
    })?;
    if !out.status.success() {
        return Err(DexError::Update(UpdateError::RefResolution {
            git_ref: refish.to_string(),
            message: "ref not found in local cache".to_string(),
        }));
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

/// Determine the target ref for an update.
///
/// Precedence: explicit `--ref`, then latest tag, then the remote's default
/// branch head (`origin/HEAD`, falling back to `HEAD`).
#[must_use]
pub fn target_refish(repo: &Path, explicit: Option<&str>) -> String {
    if let Some(r) = explicit {
        return r.to_string();
    }
    if let Some(tag) = latest_tag(repo) {
        return tag;
    }
    for candidate in ["origin/HEAD", "origin/main", "origin/master", "HEAD"] {
        if resolve_ref_sha(repo, candidate).is_ok() {
            return candidate.to_string();
        }
    }
    "HEAD".to_string()
}

/// Materialize `refish` from the cache clone into a scratch worktree and load
/// the named template from it. The worktree is removed before returning.
pub fn load_template_at_ref(
    repo: &Path,
    name: &str,
    refish: &str,
) -> Result<RemoteRevision, DexError> {
    let sha = resolve_ref_sha(repo, refish)?;

    // `git worktree add` wants a non-existent path; put it under a temp dir we
    // own so cleanup is guaranteed even if `worktree remove` fails.
    let holder = tempfile::tempdir().map_err(|source| DexError::Io {
        path: repo.to_path_buf(),
        source,
    })?;
    let wt = holder.path().join("wt");

    let added = git(
        repo,
        &["worktree", "add", "--detach", &wt.to_string_lossy(), &sha],
    )
    .map(|o| o.status.success())
    .unwrap_or(false);
    if !added {
        return Err(DexError::Update(UpdateError::RefResolution {
            git_ref: refish.to_string(),
            message: format!("could not create a worktree at {sha}"),
        }));
    }

    let guard = WorktreeGuard {
        repo: repo.to_path_buf(),
        path: wt.clone(),
    };

    let template = load_template(&TemplateSource::Directory(wt.clone()), name)?;
    drop(guard); // remove the worktree now that files are in memory

    Ok(RemoteRevision { template, sha })
}

/// Removes a git worktree on drop (and prunes the administrative entry).
struct WorktreeGuard {
    repo: PathBuf,
    path: PathBuf,
}

impl Drop for WorktreeGuard {
    fn drop(&mut self) {
        let _ = git(
            &self.repo,
            &[
                "worktree",
                "remove",
                "--force",
                &self.path.to_string_lossy(),
            ],
        );
        // Best-effort prune in case the directory was already gone.
        let _ = git(&self.repo, &["worktree", "prune"]);
    }
}

fn git(repo: &Path, args: &[&str]) -> std::io::Result<std::process::Output> {
    Command::new("git").arg("-C").arg(repo).args(args).output()
}
