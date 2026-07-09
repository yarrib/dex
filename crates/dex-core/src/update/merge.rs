//! Per-file 3-way merge between two rendered template trees and the working
//! tree.
//!
//! The merge compares three inputs for each path:
//! - **base** (`A`): the template rendered at the *old* ref,
//! - **theirs** (`B`): the template rendered at the *new* ref,
//! - **ours** (`W`): the file currently on disk in the project.
//!
//! Files present in the working tree but in *neither* `A` nor `B` are never
//! touched — that is the guarantee that local edits elsewhere never conflict.
//!
//! The overriding invariant: no branch replaces `W` unless `W == A` (the user
//! never edited it) or the replacement content embeds `W` (a clean merge or
//! git-style conflict markers). Local edits are never silently dropped.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use diffy::{ConflictStyle, MergeOptions};

use crate::error::DexError;
use crate::scaffold::RenderedTree;

/// What the merge decided to do with a single file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileAction {
    /// Template left this file unchanged (`A == B`), or the working tree
    /// already matches the target — nothing to do.
    Unchanged,
    /// New file introduced by the template; written verbatim.
    Added,
    /// Template added a file that already exists locally with different
    /// content; conflict markers are written.
    AddConflict,
    /// Template changed a file the user hadn't touched (`W == A`); replaced
    /// with the new version.
    Updated,
    /// Both the template and the user changed the file; merged cleanly.
    Merged,
    /// Both changed the file and the changes overlap; conflict markers written.
    Conflicted,
    /// Template removed a file the user hadn't touched; deleted from disk.
    Deleted,
    /// Template removed a file the user had modified; kept as-is (never
    /// silently deleted).
    DeleteConflict,
    /// Template changed a file the user had deleted; the deletion is respected.
    LocallyDeleted,
    /// Template changed a file whose local copy isn't valid UTF-8; kept as-is.
    BinaryConflict,
}

impl FileAction {
    /// True when this action left an unresolved conflict for the user.
    #[must_use]
    pub fn is_conflict(&self) -> bool {
        matches!(
            self,
            FileAction::AddConflict
                | FileAction::Conflicted
                | FileAction::DeleteConflict
                | FileAction::BinaryConflict
        )
    }
}

/// A planned change to one file. `content` is the bytes to write (`Some`) or,
/// for [`FileAction::Deleted`], `None` (the file is removed). Actions that
/// leave the working tree untouched carry `content: None`.
#[derive(Debug, Clone)]
pub struct FileChange {
    pub path: PathBuf,
    pub action: FileAction,
    pub content: Option<String>,
}

/// Merge the delta between two rendered template trees into the working tree.
///
/// Returns the full plan (one [`FileChange`] per affected path) without
/// touching the filesystem — apply it separately so `--dry-run` can preview.
pub fn merge_trees(
    old: &RenderedTree,
    new: &RenderedTree,
    project_dir: &Path,
) -> Result<Vec<FileChange>, DexError> {
    let paths: BTreeSet<&PathBuf> = old.files.keys().chain(new.files.keys()).collect();

    let mut changes = Vec::new();
    for path in paths {
        let a = old.files.get(path);
        let b = new.files.get(path);
        let w = read_working(&project_dir.join(path))?;
        changes.push(classify(path.clone(), a, b, w.as_deref()));
    }

    Ok(changes)
}

fn read_working(path: &Path) -> Result<Option<Vec<u8>>, DexError> {
    match std::fs::read(path) {
        Ok(bytes) => Ok(Some(bytes)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(source) => Err(DexError::Io {
            path: path.to_path_buf(),
            source,
        }),
    }
}

/// Classify one path given the base (`a`), theirs (`b`), and ours (`w`).
fn classify(path: PathBuf, a: Option<&String>, b: Option<&String>, w: Option<&[u8]>) -> FileChange {
    let action_and_content = match (a, b) {
        // Present in neither tree can't happen — the path came from A ∪ B.
        (None, None) => (FileAction::Unchanged, None),

        // Template left this file identical across refs.
        (Some(a), Some(b)) if a == b => (FileAction::Unchanged, None),

        // Added upstream (absent in the old render).
        (None, Some(b)) => match w {
            None => (FileAction::Added, Some(b.clone())),
            Some(w) if w == b.as_bytes() => (FileAction::Unchanged, None),
            Some(w) => match std::str::from_utf8(w) {
                Ok(w_str) => {
                    // No common ancestor: merge against an empty base so the
                    // user's file and the new file both appear in the markers.
                    let merged = three_way_merge("", w_str, b);
                    (FileAction::AddConflict, Some(merged))
                }
                Err(_) => (FileAction::BinaryConflict, None),
            },
        },

        // Removed upstream (absent in the new render).
        (Some(a), None) => match w {
            None => (FileAction::Unchanged, None),
            Some(w) if w == a.as_bytes() => (FileAction::Deleted, None),
            Some(_) => (FileAction::DeleteConflict, None),
        },

        // Modified upstream (present in both, but different).
        (Some(a), Some(b)) => match w {
            None => (FileAction::LocallyDeleted, None),
            Some(w) if w == a.as_bytes() => (FileAction::Updated, Some(b.clone())),
            Some(w) if w == b.as_bytes() => (FileAction::Unchanged, None),
            Some(w) => match std::str::from_utf8(w) {
                Ok(w_str) => match MergeOptions::new()
                    .set_conflict_style(ConflictStyle::Merge)
                    .merge(a, w_str, b)
                {
                    Ok(merged) => (FileAction::Merged, Some(merged)),
                    Err(conflicted) => (FileAction::Conflicted, Some(conflicted)),
                },
                Err(_) => (FileAction::BinaryConflict, None),
            },
        },
    };

    FileChange {
        path,
        action: action_and_content.0,
        content: action_and_content.1,
    }
}

/// 3-way merge with standard git-style conflict markers, always returning the
/// merged text (clean or with markers).
fn three_way_merge(base: &str, ours: &str, theirs: &str) -> String {
    match MergeOptions::new()
        .set_conflict_style(ConflictStyle::Merge)
        .merge(base, ours, theirs)
    {
        Ok(merged) | Err(merged) => merged,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn tree(pairs: &[(&str, &str)]) -> RenderedTree {
        let mut files = BTreeMap::new();
        for (p, c) in pairs {
            files.insert(PathBuf::from(p), (*c).to_string());
        }
        RenderedTree { files }
    }

    /// Run a merge in a temp dir seeded with the given working-tree files, and
    /// return the single change for `path`.
    fn run_case(
        old: &[(&str, &str)],
        new: &[(&str, &str)],
        working: &[(&str, &str)],
        path: &str,
    ) -> (FileChange, tempfile::TempDir) {
        let dir = tempfile::tempdir().unwrap();
        for (p, c) in working {
            let full = dir.path().join(p);
            if let Some(parent) = full.parent() {
                std::fs::create_dir_all(parent).unwrap();
            }
            std::fs::write(full, c).unwrap();
        }
        let changes = merge_trees(&tree(old), &tree(new), dir.path()).unwrap();
        let change = changes
            .into_iter()
            .find(|c| c.path == Path::new(path))
            .unwrap_or(FileChange {
                path: PathBuf::from(path),
                action: FileAction::Unchanged,
                content: None,
            });
        (change, dir)
    }

    #[test]
    fn template_unchanged_is_noop() {
        let (c, _d) = run_case(
            &[("a.txt", "same\n")],
            &[("a.txt", "same\n")],
            &[("a.txt", "locally edited\n")],
            "a.txt",
        );
        assert_eq!(c.action, FileAction::Unchanged);
        assert!(c.content.is_none());
    }

    #[test]
    fn added_upstream_absent_locally_is_added() {
        let (c, _d) = run_case(&[], &[("new.txt", "brand new\n")], &[], "new.txt");
        assert_eq!(c.action, FileAction::Added);
        assert_eq!(c.content.as_deref(), Some("brand new\n"));
    }

    #[test]
    fn added_upstream_identical_locally_is_noop() {
        let (c, _d) = run_case(
            &[],
            &[("new.txt", "brand new\n")],
            &[("new.txt", "brand new\n")],
            "new.txt",
        );
        assert_eq!(c.action, FileAction::Unchanged);
    }

    #[test]
    fn added_upstream_different_locally_is_add_conflict() {
        let (c, _d) = run_case(
            &[],
            &[("new.txt", "upstream\n")],
            &[("new.txt", "mine\n")],
            "new.txt",
        );
        assert_eq!(c.action, FileAction::AddConflict);
        let body = c.content.unwrap();
        assert!(body.contains("<<<<<<<"), "missing conflict markers: {body}");
        assert!(body.contains("mine"), "local content dropped: {body}");
        assert!(
            body.contains("upstream"),
            "upstream content dropped: {body}"
        );
    }

    #[test]
    fn deleted_upstream_unmodified_locally_is_deleted() {
        let (c, _d) = run_case(
            &[("gone.txt", "bye\n")],
            &[],
            &[("gone.txt", "bye\n")],
            "gone.txt",
        );
        assert_eq!(c.action, FileAction::Deleted);
        assert!(c.content.is_none());
    }

    #[test]
    fn deleted_upstream_modified_locally_is_delete_conflict() {
        let (c, _d) = run_case(
            &[("keep.txt", "orig\n")],
            &[],
            &[("keep.txt", "i changed this\n")],
            "keep.txt",
        );
        assert_eq!(c.action, FileAction::DeleteConflict);
        // Never carries content to write — the local file is left untouched.
        assert!(c.content.is_none());
    }

    #[test]
    fn deleted_upstream_already_gone_is_noop() {
        let (c, _d) = run_case(&[("gone.txt", "bye\n")], &[], &[], "gone.txt");
        assert_eq!(c.action, FileAction::Unchanged);
    }

    #[test]
    fn modified_upstream_unmodified_locally_is_updated() {
        let (c, _d) = run_case(
            &[("f.txt", "v1\n")],
            &[("f.txt", "v2\n")],
            &[("f.txt", "v1\n")],
            "f.txt",
        );
        assert_eq!(c.action, FileAction::Updated);
        assert_eq!(c.content.as_deref(), Some("v2\n"));
    }

    #[test]
    fn modified_upstream_already_at_target_is_noop() {
        let (c, _d) = run_case(
            &[("f.txt", "v1\n")],
            &[("f.txt", "v2\n")],
            &[("f.txt", "v2\n")],
            "f.txt",
        );
        assert_eq!(c.action, FileAction::Unchanged);
    }

    #[test]
    fn modified_upstream_locally_deleted_is_respected() {
        let (c, _d) = run_case(&[("f.txt", "v1\n")], &[("f.txt", "v2\n")], &[], "f.txt");
        assert_eq!(c.action, FileAction::LocallyDeleted);
        assert!(c.content.is_none());
    }

    #[test]
    fn non_overlapping_edits_merge_cleanly() {
        // Upstream edits the first line, user edits the last line.
        let old = "line1\nline2\nline3\n";
        let new = "LINE1\nline2\nline3\n";
        let mine = "line1\nline2\nLINE3\n";
        let (c, _d) = run_case(
            &[("f.txt", old)],
            &[("f.txt", new)],
            &[("f.txt", mine)],
            "f.txt",
        );
        assert_eq!(c.action, FileAction::Merged);
        let body = c.content.unwrap();
        assert!(body.contains("LINE1"), "upstream edit lost: {body}");
        assert!(body.contains("LINE3"), "local edit lost: {body}");
        assert!(!body.contains("<<<<<<<"), "unexpected conflict: {body}");
    }

    #[test]
    fn overlapping_edits_produce_conflict_markers() {
        let old = "shared line\n";
        let new = "upstream version\n";
        let mine = "my version\n";
        let (c, _d) = run_case(
            &[("f.txt", old)],
            &[("f.txt", new)],
            &[("f.txt", mine)],
            "f.txt",
        );
        assert_eq!(c.action, FileAction::Conflicted);
        let body = c.content.unwrap();
        assert!(body.contains("<<<<<<<"), "missing markers: {body}");
        assert!(body.contains("======="), "missing separator: {body}");
        assert!(body.contains(">>>>>>>"), "missing closing marker: {body}");
        assert!(body.contains("my version"), "local edit dropped: {body}");
        assert!(
            body.contains("upstream version"),
            "upstream dropped: {body}"
        );
    }

    #[test]
    fn modified_upstream_binary_local_is_binary_conflict() {
        let dir = tempfile::tempdir().unwrap();
        // Invalid UTF-8 working-tree file.
        std::fs::write(dir.path().join("f.bin"), [0xff, 0xfe, 0x00]).unwrap();
        let old = tree(&[("f.bin", "v1\n")]);
        let new = tree(&[("f.bin", "v2\n")]);
        let changes = merge_trees(&old, &new, dir.path()).unwrap();
        let c = changes.into_iter().next().unwrap();
        assert_eq!(c.action, FileAction::BinaryConflict);
        assert!(c.content.is_none());
        // Local bytes must survive untouched (apply writes nothing).
        assert_eq!(
            std::fs::read(dir.path().join("f.bin")).unwrap(),
            vec![0xff, 0xfe, 0x00]
        );
    }

    #[test]
    fn never_silently_drops_local_edits() {
        // For every content-bearing change, the local file's bytes must be
        // either untouched (content is None) or embedded in the written text.
        type Pairs = &'static [(&'static str, &'static str)];
        let cases: &[(Pairs, Pairs, Pairs)] = &[
            (&[], &[("x", "up\n")], &[("x", "mine\n")]), // add-conflict
            (&[("x", "v1\n")], &[("x", "v2\n")], &[("x", "v1x\n")]), // real merge/conflict
        ];
        for (old, new, working) in cases {
            let (c, _d) = run_case(old, new, working, "x");
            let local = working
                .iter()
                .find(|(p, _)| *p == "x")
                .map(|(_, c)| c.to_string())
                .unwrap();
            if let Some(written) = &c.content {
                let local_body = local.trim_end();
                assert!(
                    written.contains(local_body),
                    "action {:?} wrote content without the local edit {local:?}: {written}",
                    c.action
                );
            }
        }
    }
}
