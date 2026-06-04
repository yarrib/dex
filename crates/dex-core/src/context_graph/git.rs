//! Thin wrapper over the `git` CLI for the project-memory engine.
//!
//! dex-core has no UI, but shelling out to git is fine — it returns structured
//! data. We avoid a libgit2 dependency to keep the binary small and the build
//! simple. Two `git log` passes mirror the project-memory-engine skill: one for
//! metadata, one for changed files.

use std::path::Path;
use std::process::Command;

use crate::error::ContextError;

/// Field separator (ASCII Unit Separator) — never appears in git output fields.
const FS: char = '\u{1f}';
/// Record separator (ASCII Record Separator) — terminates each commit record.
const RS: char = '\u{1e}';

/// A commit as read from git history, before classification.
#[derive(Debug, Clone)]
pub struct RawCommit {
    /// Full 40-char commit hash.
    pub sha: String,
    /// Abbreviated 7-char hash, used in node filenames and wikilinks.
    pub short_sha: String,
    pub author: String,
    /// Author date, `YYYY-MM-DD`.
    pub date: String,
    pub subject: String,
    pub body: String,
    /// Paths changed by this commit (POSIX separators, repo-relative).
    pub files: Vec<String>,
}

/// Verify `root` is inside a git work tree before we try to read history.
pub fn ensure_repo(root: &Path) -> Result<(), ContextError> {
    let out = Command::new("git")
        .current_dir(root)
        .args(["rev-parse", "--is-inside-work-tree"])
        .output()
        .map_err(|e| ContextError::GitSpawn(e.to_string()))?;

    if out.status.success() && String::from_utf8_lossy(&out.stdout).trim() == "true" {
        Ok(())
    } else {
        Err(ContextError::NotARepo(root.display().to_string()))
    }
}

/// Read commit history oldest-first, with changed files attached.
///
/// Merge commits are skipped (`--no-merges`); `limit` caps the number of most
/// recent commits considered (useful for a first run on a large repo).
pub fn log(root: &Path, limit: Option<usize>) -> Result<Vec<RawCommit>, ContextError> {
    let meta = run_log(
        root,
        limit,
        &format!("--pretty=format:%H{FS}%an{FS}%ad{FS}%s{FS}%b{RS}"),
        true,
    )?;
    let files_raw = run_log(root, limit, &format!("--pretty=format:{RS}%H"), false)?;

    let files_by_sha = parse_files(&files_raw);

    let mut commits: Vec<RawCommit> = meta
        .split(RS)
        .filter(|rec| !rec.trim().is_empty())
        .filter_map(|rec| parse_record(rec, &files_by_sha))
        .collect();

    // git logs newest-first; reverse so "prior nodes" are simply earlier in the
    // vector when we stitch edges.
    commits.reverse();
    Ok(commits)
}

fn run_log(
    root: &Path,
    limit: Option<usize>,
    pretty: &str,
    date_short: bool,
) -> Result<String, ContextError> {
    let mut args: Vec<String> = vec!["log".into(), "--no-merges".into()];
    if date_short {
        args.push("--date=short".into());
    } else {
        args.push("--name-only".into());
    }
    if let Some(n) = limit {
        args.push(format!("-n{n}"));
    }
    args.push(pretty.into());

    let out = Command::new("git")
        .current_dir(root)
        .args(&args)
        .output()
        .map_err(|e| ContextError::GitSpawn(e.to_string()))?;

    if !out.status.success() {
        return Err(ContextError::Git(
            String::from_utf8_lossy(&out.stderr).trim().to_string(),
        ));
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

fn parse_record(rec: &str, files_by_sha: &[(String, Vec<String>)]) -> Option<RawCommit> {
    let mut fields = rec.trim_start_matches('\n').splitn(5, FS);
    let sha = fields.next()?.trim().to_string();
    if sha.len() < 7 {
        return None;
    }
    let author = fields.next().unwrap_or_default().to_string();
    let date = fields.next().unwrap_or_default().to_string();
    let subject = fields.next().unwrap_or_default().to_string();
    let body = fields.next().unwrap_or_default().trim().to_string();
    let short_sha = sha[..7].to_string();

    let files = files_by_sha
        .iter()
        .find(|(s, _)| *s == sha)
        .map(|(_, f)| f.clone())
        .unwrap_or_default();

    Some(RawCommit {
        sha,
        short_sha,
        author,
        date,
        subject,
        body,
        files,
    })
}

/// Parse the `--name-only` pass into (sha, files) pairs.
fn parse_files(raw: &str) -> Vec<(String, Vec<String>)> {
    raw.split(RS)
        .filter(|c| !c.trim().is_empty())
        .filter_map(|chunk| {
            let mut lines = chunk.lines().filter(|l| !l.trim().is_empty());
            let sha = lines.next()?.trim().to_string();
            let files: Vec<String> = lines.map(|l| l.trim().to_string()).collect();
            Some((sha, files))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_files_groups_by_commit() {
        let raw = format!("{RS}abc123\nsrc/a.rs\nsrc/b.rs\n{RS}def456\nREADME.md");
        let parsed = parse_files(&raw);
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0].0, "abc123");
        assert_eq!(parsed[0].1, vec!["src/a.rs", "src/b.rs"]);
        assert_eq!(parsed[1].1, vec!["README.md"]);
    }

    #[test]
    fn parse_record_reads_all_fields() {
        let files = vec![(
            "abcdef1234567890".to_string(),
            vec!["src/main.rs".to_string()],
        )];
        let rec = format!("abcdef1234567890{FS}Ada{FS}2026-01-02{FS}feat: add thing{FS}body text");
        let c = parse_record(&rec, &files).unwrap();
        assert_eq!(c.short_sha, "abcdef1");
        assert_eq!(c.author, "Ada");
        assert_eq!(c.date, "2026-01-02");
        assert_eq!(c.subject, "feat: add thing");
        assert_eq!(c.body, "body text");
        assert_eq!(c.files, vec!["src/main.rs"]);
    }
}
