//! Markdown rendering for the project-memory graph: per-commit node files,
//! the `INDEX.md` city map, and the static `USER_MANUAL.md`.
//!
//! All output is plain Markdown with `[[wikilinks]]`, so it renders as a live
//! graph in Obsidian, Logseq, or VS Code (Foam) with no build step.

use std::collections::BTreeMap;
use std::path::Path;
use std::sync::OnceLock;

use regex::Regex;

use crate::error::DexError;

use super::{EdgeKind, ExportOptions, ExportReport, FunctionalArea, Node, SyncOptions, SyncReport};

/// Write the whole graph: nodes (incremental), INDEX.md, and USER_MANUAL.md.
pub fn write_graph(
    wiki_dir: &Path,
    manual_path: &Path,
    index_path: &Path,
    nodes: &[Node],
    opts: &SyncOptions,
) -> Result<SyncReport, DexError> {
    mkdir(wiki_dir)?;

    if opts.rebuild {
        clear_nodes(wiki_dir)?;
    }

    // Write node files — skip those already on disk unless rebuilding.
    let mut nodes_written = 0usize;
    for node in nodes {
        let path = wiki_dir.join(format!("{}.md", node.stem));
        if opts.rebuild || !path.exists() {
            write_file(&path, &render_node(node))?;
            nodes_written += 1;
        }
    }

    // INDEX.md is always rewritten so it reflects the full graph.
    write_file(index_path, &render_index(nodes))?;

    // USER_MANUAL.md is written once (or on rebuild) — it's reference docs.
    let manual_written = if opts.rebuild || !manual_path.exists() {
        write_file(manual_path, USER_MANUAL)?;
        true
    } else {
        false
    };

    let mut area_counts: Vec<(FunctionalArea, usize)> = FunctionalArea::ALL
        .iter()
        .map(|a| (*a, nodes.iter().filter(|n| n.area == *a).count()))
        .collect();
    area_counts.retain(|(_, c)| *c > 0);

    Ok(SyncReport {
        wiki_dir: wiki_dir.to_path_buf(),
        index_path: index_path.to_path_buf(),
        manual_path: manual_path.to_path_buf(),
        nodes_total: nodes.len(),
        nodes_written,
        manual_written,
        area_counts,
    })
}

// --- Node rendering ---------------------------------------------------------

fn render_node(node: &Node) -> String {
    let c = &node.commit;
    let mut s = String::new();

    // Frontmatter — useful in Obsidian/Dataview and for the skill to parse.
    s.push_str("---\n");
    s.push_str(&format!("sha: {}\n", c.sha));
    s.push_str(&format!("short_sha: {}\n", c.short_sha));
    s.push_str(&format!("author: {}\n", c.author));
    s.push_str(&format!("date: {}\n", c.date));
    s.push_str(&format!("class: {}\n", node.class.label()));
    s.push_str(&format!("area: {}\n", node.area.title()));
    s.push_str(&format!("tags: [{}]\n", node.class.tag()));
    s.push_str("---\n\n");

    s.push_str(&format!("# {} {}\n\n", node.class.label(), c.subject));
    s.push_str(&format!(
        "**Commit:** `{}` · **Author:** {} · **Date:** {} · **Area:** {}\n\n",
        c.short_sha,
        c.author,
        c.date,
        node.area.title()
    ));

    if c.body.is_empty() {
        s.push_str("_No extended commit description._\n\n");
    } else {
        s.push_str(&c.body);
        s.push_str("\n\n");
    }

    s.push_str("## Changed files\n\n");
    if node.signal_files.is_empty() {
        s.push_str("_No tracked source files (vendored/lock changes only)._\n\n");
    } else {
        let shown = node.signal_files.iter().take(20);
        for f in shown {
            s.push_str(&format!("- `{f}`\n"));
        }
        if node.signal_files.len() > 20 {
            s.push_str(&format!("- _…and {} more_\n", node.signal_files.len() - 20));
        }
        s.push('\n');
    }

    s.push_str("## Relationships\n\n");
    let has_rel = !node.edges.is_empty() || !node.issue_refs.is_empty();
    if !has_rel {
        s.push_str("_No linked nodes yet._\n");
    } else {
        for kind in [
            EdgeKind::InfluencedBy,
            EdgeKind::ModifiedBy,
            EdgeKind::ImplementedIn,
            EdgeKind::CoOccurrence,
        ] {
            for e in node.edges.iter().filter(|e| e.kind == kind) {
                let note = e
                    .note
                    .as_ref()
                    .map(|n| format!(" ({n})"))
                    .unwrap_or_default();
                s.push_str(&format!(
                    "- **{}** → [[{}]]{}\n",
                    e.kind.wikilink_label(),
                    e.target_stem,
                    note
                ));
            }
        }
        if !node.issue_refs.is_empty() {
            s.push_str(&format!(
                "- **resolved-by** → {} _(this commit)_\n",
                node.issue_refs
                    .iter()
                    .map(|r| format!("`{r}`"))
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
    }

    s
}

// --- INDEX rendering --------------------------------------------------------

fn render_index(nodes: &[Node]) -> String {
    let mut s = String::new();
    s.push_str("# Project Memory — Index\n\n");
    s.push_str(&format!(
        "> A Project Evolution Knowledge Graph of **{}** commits. \
Generated by `dex context sync` (the Rust engine) and refreshable via the \
`/project-memory-engine` skill. See [USER_MANUAL](../USER_MANUAL.md) for how to read it.\n\n",
        nodes.len()
    ));

    s.push_str("## Node types\n\n");
    s.push_str("| Type | Meaning |\n|------|---------|\n");
    s.push_str("| `[Decision]` | Architectural pivot |\n");
    s.push_str("| `[Evolution]` | Major feature / capability addition |\n");
    s.push_str("| `[Stability]` | Bug fix, hardening, resilience |\n");
    s.push_str("| `[Dependency]` | Environment / config / packaging |\n\n");

    s.push_str("## Edge types\n\n");
    s.push_str("| Edge | Meaning |\n|------|---------|\n");
    s.push_str("| `[[influenced-by]]` | Builds on a prior decision |\n");
    s.push_str("| `[[modified-by]]` | Alters a design pattern (usually a fix) |\n");
    s.push_str("| `[[implemented-in]]` | Connects a design doc to the code realising it |\n");
    s.push_str("| `[[co-occurrence]]` | Files that consistently change together |\n");
    s.push_str("| `resolved-by` | Connects an Issue / PR ID to the change |\n\n");

    s.push_str("## Reading order for new agents\n\n");
    s.push_str(
        "1. Skim the areas below top-to-bottom — they're ordered most-foundational first.\n\
2. Open the `[Decision]` nodes in **Foundation & Architecture** to learn the ground rules.\n\
3. Follow `[[influenced-by]]` edges forward to see how features built on those decisions.\n\
4. Before changing a module, open its most recent node and check its `[[co-occurrence]]` cluster.\n\n",
    );

    s.push_str("## Knowledge map\n\n");
    for (i, area) in FunctionalArea::ALL.iter().enumerate() {
        s.push_str(&format!("### {}. {}\n\n", i + 1, area.title()));
        let mut in_area: Vec<&Node> = nodes.iter().filter(|n| n.area == *area).collect();
        if in_area.is_empty() {
            s.push_str("_No nodes yet._\n\n");
            continue;
        }
        // Newest first within an area.
        in_area.reverse();
        for n in in_area {
            s.push_str(&format!(
                "- `{}` **{}** {} — [[{}]] _( {} )_\n",
                n.commit.short_sha,
                n.class.label(),
                escape_pipe(&n.commit.subject),
                n.stem,
                n.commit.date
            ));
        }
        s.push('\n');
    }

    // Co-change clusters: the most frequently co-changing file pairs.
    let clusters = top_co_change_pairs(nodes, 8);
    if !clusters.is_empty() {
        s.push_str("## Co-change clusters\n\n");
        s.push_str("Files that most frequently change together (coupling hotspots):\n\n");
        for (a, b, count) in clusters {
            s.push_str(&format!("- `{a}` + `{b}` — {count} commits\n"));
        }
        s.push('\n');
    }

    s.push_str("## Refreshing\n\n");
    s.push_str(
        "Run `dex context sync` (incremental) or `dex context sync --rebuild` (full). \
The `/project-memory-engine` skill wraps this and can add semantic enrichment.\n",
    );

    s
}

/// Compute the top-N most frequently co-changing file pairs across all nodes.
fn top_co_change_pairs(nodes: &[Node], n: usize) -> Vec<(String, String, usize)> {
    let mut pairs: BTreeMap<(String, String), usize> = BTreeMap::new();
    for node in nodes {
        let files = &node.signal_files;
        for i in 0..files.len() {
            for j in (i + 1)..files.len() {
                let (a, b) = if files[i] <= files[j] {
                    (files[i].clone(), files[j].clone())
                } else {
                    (files[j].clone(), files[i].clone())
                };
                *pairs.entry((a, b)).or_insert(0) += 1;
            }
        }
    }
    let mut v: Vec<(String, String, usize)> = pairs
        .into_iter()
        .filter(|(_, c)| *c >= 2)
        .map(|((a, b), c)| (a, b, c))
        .collect();
    // Most-coupled first; stable tie-break on names.
    v.sort_by(|x, y| y.2.cmp(&x.2).then(x.0.cmp(&y.0)).then(x.1.cmp(&y.1)));
    v.truncate(n);
    v
}

fn escape_pipe(s: &str) -> String {
    s.replace('|', "\\|")
}

// --- Filesystem helpers -----------------------------------------------------

fn mkdir(dir: &Path) -> Result<(), DexError> {
    std::fs::create_dir_all(dir).map_err(|source| DexError::Io {
        path: dir.to_path_buf(),
        source,
    })
}

fn write_file(path: &Path, content: &str) -> Result<(), DexError> {
    if let Some(parent) = path.parent() {
        mkdir(parent)?;
    }
    std::fs::write(path, content).map_err(|source| DexError::Io {
        path: path.to_path_buf(),
        source,
    })
}

/// Remove generated node files and INDEX.md (for `--rebuild`). Leaves
/// `.obsidian/`, hand-authored notes, and other non-node files alone.
fn clear_nodes(wiki_dir: &Path) -> Result<(), DexError> {
    let entries = match std::fs::read_dir(wiki_dir) {
        Ok(e) => e,
        Err(_) => return Ok(()),
    };
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name == "INDEX.md" || is_node_filename(&name) {
            let _ = std::fs::remove_file(entry.path());
        }
    }
    Ok(())
}

/// A node file looks like `<7-hex>-<slug>.md`.
fn is_node_filename(name: &str) -> bool {
    let Some(stem) = name.strip_suffix(".md") else {
        return false;
    };
    let Some((sha, _rest)) = stem.split_once('-') else {
        return false;
    };
    sha.len() == 7 && sha.chars().all(|c| c.is_ascii_hexdigit())
}

// --- mdBook export ----------------------------------------------------------

const SUMMARY_START: &str = "<!-- project-memory:start -->";
const SUMMARY_END: &str = "<!-- project-memory:end -->";

/// Render the committed wiki into mdBook-ready pages under `opts.out_dir` and
/// optionally inject a navigation section into a `SUMMARY.md`.
pub fn write_export(
    wiki_dir: &Path,
    manual_path: &Path,
    nodes: &[Node],
    opts: &ExportOptions,
) -> Result<ExportReport, DexError> {
    mkdir(&opts.out_dir)?;
    let mut pages_written = 0usize;

    // Index (rewrite the manual link to sit alongside in the export dir).
    if let Some(index) = read_opt(&wiki_dir.join("INDEX.md"))? {
        let body = rewrite_links(&index).replace("](../USER_MANUAL.md)", "](USER_MANUAL.md)");
        write_file(&opts.out_dir.join("INDEX.md"), &body)?;
        pages_written += 1;
    }

    // Manual.
    if let Some(manual) = read_opt(manual_path)? {
        write_file(
            &opts.out_dir.join("USER_MANUAL.md"),
            &rewrite_links(&manual),
        )?;
        pages_written += 1;
    }

    // Node pages — only those present on disk (mirrors the committed graph).
    for node in nodes {
        let src = wiki_dir.join(format!("{}.md", node.stem));
        if let Some(content) = read_opt(&src)? {
            write_file(
                &opts.out_dir.join(format!("{}.md", node.stem)),
                &rewrite_links(&content),
            )?;
            pages_written += 1;
        }
    }

    let summary_updated = if let Some(summary) = &opts.summary_path {
        let dir_name = opts
            .out_dir
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| "wiki".to_string());
        inject_summary(summary, &build_summary_section(nodes, wiki_dir, &dir_name))?;
        true
    } else {
        false
    };

    Ok(ExportReport {
        out_dir: opts.out_dir.clone(),
        pages_written,
        summary_updated,
    })
}

/// Rewrite Obsidian `[[stem]]` / `[[stem|text]]` links into relative mdBook
/// links (`[text](stem.md)`), since the export keeps every page in one folder.
fn rewrite_links(content: &str) -> String {
    static RE: OnceLock<Regex> = OnceLock::new();
    let re = RE.get_or_init(|| Regex::new(r"\[\[([^\]|]+?)(?:\|([^\]]+))?\]\]").unwrap());
    re.replace_all(content, |caps: &regex::Captures| {
        let target = caps[1].trim();
        let text = caps.get(2).map(|m| m.as_str().trim()).unwrap_or(target);
        format!("[{text}]({target}.md)")
    })
    .into_owned()
}

/// Build the `SUMMARY.md` section (between markers): manual + index + node pages
/// grouped by functional area, newest-first.
fn build_summary_section(nodes: &[Node], wiki_dir: &Path, dir_name: &str) -> String {
    let mut s = String::new();
    s.push_str(SUMMARY_START);
    s.push('\n');
    s.push_str("# Project Memory\n\n");
    s.push_str(&format!("- [How to read it]({dir_name}/USER_MANUAL.md)\n"));
    s.push_str(&format!("- [Knowledge Map]({dir_name}/INDEX.md)\n"));

    for area in FunctionalArea::ALL {
        let mut in_area: Vec<&Node> = nodes
            .iter()
            .filter(|n| n.area == area && wiki_dir.join(format!("{}.md", n.stem)).exists())
            .collect();
        if in_area.is_empty() {
            continue;
        }
        in_area.reverse(); // newest first
        // Draft chapter (empty link) acts as a non-clickable group header.
        s.push_str(&format!("  - [{}]()\n", area.title()));
        for n in in_area {
            s.push_str(&format!(
                "    - [`{}` {}]({}/{}.md)\n",
                n.commit.short_sha,
                link_text(&n.commit.subject),
                dir_name,
                n.stem
            ));
        }
    }

    s.push_str(SUMMARY_END);
    s
}

/// Replace markers in `summary_path` with `section`. If markers are absent,
/// append the section (with a leading blank line) to the end of the file.
fn inject_summary(summary_path: &Path, section: &str) -> Result<(), DexError> {
    let existing = read_opt(summary_path)?.unwrap_or_default();

    let new = if let (Some(start), Some(end)) =
        (existing.find(SUMMARY_START), existing.find(SUMMARY_END))
    {
        let end = end + SUMMARY_END.len();
        format!("{}{}{}", &existing[..start], section, &existing[end..])
    } else {
        format!("{}\n\n{}\n", existing.trim_end(), section)
    };

    write_file(summary_path, &new)
}

/// Sanitize a commit subject for use as Markdown link text.
fn link_text(subject: &str) -> String {
    subject.replace(['[', ']'], "")
}

fn read_opt(path: &Path) -> Result<Option<String>, DexError> {
    match std::fs::read_to_string(path) {
        Ok(s) => Ok(Some(s)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(source) => Err(DexError::Io {
            path: path.to_path_buf(),
            source,
        }),
    }
}

// --- Static user manual -----------------------------------------------------

/// The `.context/USER_MANUAL.md` content. Adapted for dex from the
/// project-memory-engine reference: explains the graph for three audiences.
const USER_MANUAL: &str = include_str!("USER_MANUAL.md");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognises_node_filenames() {
        assert!(is_node_filename("a1b2c3d-add-thing.md"));
        assert!(!is_node_filename("INDEX.md"));
        assert!(!is_node_filename("USER_MANUAL.md"));
        assert!(!is_node_filename("notes.md"));
    }

    #[test]
    fn user_manual_is_embedded() {
        assert!(USER_MANUAL.contains("Project Memory"));
    }

    #[test]
    fn rewrite_links_converts_wikilinks() {
        assert_eq!(
            rewrite_links("see [[abc1234-add-thing]] now"),
            "see [abc1234-add-thing](abc1234-add-thing.md) now"
        );
        assert_eq!(
            rewrite_links("see [[abc1234-add-thing|the change]]"),
            "see [the change](abc1234-add-thing.md)"
        );
        assert_eq!(rewrite_links("no links here"), "no links here");
    }

    #[test]
    fn inject_summary_replaces_between_markers() {
        let dir = tempfile::tempdir().unwrap();
        let summary = dir.path().join("SUMMARY.md");
        std::fs::write(
            &summary,
            format!("# Summary\n\n- [Home](index.md)\n\n{SUMMARY_START}\n{SUMMARY_END}\n"),
        )
        .unwrap();

        inject_summary(
            &summary,
            &format!("{SUMMARY_START}\n# Project Memory\n{SUMMARY_END}"),
        )
        .unwrap();
        let out = std::fs::read_to_string(&summary).unwrap();
        assert!(out.contains("- [Home](index.md)"));
        assert!(out.contains("# Project Memory"));
        // Markers must not be duplicated.
        assert_eq!(out.matches(SUMMARY_START).count(), 1);
    }
}
