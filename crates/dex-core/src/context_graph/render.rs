//! Markdown rendering for the project-memory graph: per-commit node files,
//! the `INDEX.md` city map, and the static `USER_MANUAL.md`.
//!
//! All output is plain Markdown with `[[wikilinks]]`, so it renders as a live
//! graph in Obsidian, Logseq, or VS Code (Foam) with no build step.

use std::collections::BTreeMap;
use std::path::Path;

use crate::error::DexError;

use super::{EdgeKind, FunctionalArea, Node, SyncOptions, SyncReport};

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
}
