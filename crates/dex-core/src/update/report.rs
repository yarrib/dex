//! Summary of an applied (or previewed) `dex update`, mirroring the shape of
//! `TraitResult` so the CLI can render it the same way.

use std::path::PathBuf;

use crate::update::merge::{FileAction, FileChange};

/// Aggregated outcome of a `dex update` plan or apply.
#[derive(Debug, Default)]
pub struct UpdateReport {
    pub old_ref: String,
    pub new_ref: String,
    /// Files the template changed and the user hadn't touched.
    pub updated: Vec<PathBuf>,
    /// New files introduced by the template.
    pub added: Vec<PathBuf>,
    /// Files where template and local edits merged cleanly.
    pub merged: Vec<PathBuf>,
    /// Files left with conflict markers (or kept because they couldn't be
    /// merged) that the user must resolve by hand.
    pub conflicts: Vec<PathBuf>,
    /// Files removed because the template removed them.
    pub deleted: Vec<PathBuf>,
    /// Human-readable notes for the situations that need attention but aren't
    /// standard conflicts (kept-on-delete, respected local deletions, binaries).
    pub notices: Vec<String>,
}

impl UpdateReport {
    /// Build a report from the merge plan.
    #[must_use]
    pub fn from_changes(old_ref: &str, new_ref: &str, changes: &[FileChange]) -> Self {
        let mut report = UpdateReport {
            old_ref: old_ref.to_string(),
            new_ref: new_ref.to_string(),
            ..Default::default()
        };

        for change in changes {
            let path = change.path.clone();
            let shown = path.display();
            match change.action {
                FileAction::Unchanged => {}
                FileAction::Added => report.added.push(path),
                FileAction::Updated => report.updated.push(path),
                FileAction::Merged => report.merged.push(path),
                FileAction::Conflicted => report.conflicts.push(path),
                FileAction::AddConflict => {
                    report.notices.push(format!(
                        "{shown}: template added a file that already exists locally — conflict markers written"
                    ));
                    report.conflicts.push(path);
                }
                FileAction::Deleted => report.deleted.push(path),
                FileAction::DeleteConflict => {
                    report.notices.push(format!(
                        "{shown}: template removed this file but you changed it — kept your version"
                    ));
                }
                FileAction::LocallyDeleted => {
                    report.notices.push(format!(
                        "{shown}: template changed this file but you deleted it — left deleted"
                    ));
                }
                FileAction::BinaryConflict => {
                    report.notices.push(format!(
                        "{shown}: template changed this file but the local copy is binary — kept your version"
                    ));
                    report.conflicts.push(path);
                }
            }
        }

        report
    }

    /// Total number of files whose contents were written or removed.
    #[must_use]
    pub fn files_changed(&self) -> usize {
        self.updated.len() + self.added.len() + self.merged.len() + self.deleted.len()
    }

    /// True when the update produced conflicts the user must resolve.
    #[must_use]
    pub fn has_conflicts(&self) -> bool {
        !self.conflicts.is_empty()
    }

    /// True when nothing at all changed.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.files_changed() == 0 && self.conflicts.is_empty() && self.notices.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn change(path: &str, action: FileAction) -> FileChange {
        FileChange {
            path: PathBuf::from(path),
            action,
            content: None,
        }
    }

    #[test]
    fn report_buckets_actions_correctly() {
        let changes = vec![
            change("added.txt", FileAction::Added),
            change("updated.txt", FileAction::Updated),
            change("merged.txt", FileAction::Merged),
            change("conflict.txt", FileAction::Conflicted),
            change("gone.txt", FileAction::Deleted),
            change("unchanged.txt", FileAction::Unchanged),
        ];

        let report = UpdateReport::from_changes("v1", "v2", &changes);
        assert_eq!(report.old_ref, "v1");
        assert_eq!(report.new_ref, "v2");
        assert_eq!(report.added, vec![PathBuf::from("added.txt")]);
        assert_eq!(report.updated, vec![PathBuf::from("updated.txt")]);
        assert_eq!(report.merged, vec![PathBuf::from("merged.txt")]);
        assert_eq!(report.conflicts, vec![PathBuf::from("conflict.txt")]);
        assert_eq!(report.deleted, vec![PathBuf::from("gone.txt")]);
        assert_eq!(report.files_changed(), 4);
        assert!(report.has_conflicts());
    }

    #[test]
    fn add_conflict_counts_as_conflict_with_notice() {
        let changes = vec![change("x", FileAction::AddConflict)];
        let report = UpdateReport::from_changes("a", "b", &changes);
        assert_eq!(report.conflicts, vec![PathBuf::from("x")]);
        assert_eq!(report.notices.len(), 1);
    }

    #[test]
    fn delete_conflict_is_notice_not_change() {
        let changes = vec![change("x", FileAction::DeleteConflict)];
        let report = UpdateReport::from_changes("a", "b", &changes);
        assert!(report.deleted.is_empty());
        assert_eq!(report.notices.len(), 1);
        assert!(!report.has_conflicts());
    }

    #[test]
    fn empty_plan_is_empty_report() {
        let report = UpdateReport::from_changes("a", "b", &[]);
        assert!(report.is_empty());
    }
}
