//! Markdown rendering for the project-memory graph: per-commit node files,
//! the `INDEX.md` city map, and the static `USER_MANUAL.md`.
//!
//! All output is plain Markdown with `[[wikilinks]]`, so it renders as a live
//! graph in Obsidian, Logseq, or VS Code (Foam) with no build step.

use std::collections::BTreeMap;
use std::path::Path;
use std::sync::OnceLock;

use regex::Regex;
use serde::Serialize;

use crate::error::DexError;

use super::{
    EdgeKind, ExportOptions, ExportReport, FunctionalArea, Node, NodeClass, SyncOptions, SyncReport,
};

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
        let body = clean_body(&c.body);
        if body.is_empty() {
            s.push_str("_No extended commit description._\n\n");
        } else {
            s.push_str(&body);
            s.push_str("\n\n");
        }
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

/// Strip AI-assistant attribution noise from a commit body so the knowledge
/// graph reads like project history, not tooling boilerplate. Removes:
///
/// - `Generated by/with [Claude Code](…)` footers (which may hard-wrap lines)
/// - bare `https://claude.ai/code/…` / `https://claude.com/claude-code` links
/// - `Co-authored-by:` trailers crediting Claude/Anthropic
///
/// then drops any now-orphaned trailing `---` rule and collapses blank runs.
fn clean_body(body: &str) -> String {
    static GENERATED: OnceLock<Regex> = OnceLock::new();
    static URL_LINE: OnceLock<Regex> = OnceLock::new();
    static COAUTHOR: OnceLock<Regex> = OnceLock::new();
    static BLANKS: OnceLock<Regex> = OnceLock::new();

    // `\s*` (which matches newlines) between "Claude" and "Code" absorbs the
    // hard wrap seen in real commit footers (`[Claude\nCode](…)`).
    let generated = GENERATED.get_or_init(|| {
        Regex::new(r"(?i)(🤖\s*)?_?\s*generated (by|with) \[claude\s*code\]\([^)]*\)_?").unwrap()
    });
    let url_line =
        URL_LINE.get_or_init(|| Regex::new(r"(?i)^https?://claude\.(ai|com)/\S*$").unwrap());
    let coauthor =
        COAUTHOR.get_or_init(|| Regex::new(r"(?i)^co-authored-by:.*(claude|anthropic)").unwrap());
    let blanks = BLANKS.get_or_init(|| Regex::new(r"\n{3,}").unwrap());

    let without_footer = generated.replace_all(body, "");

    let kept: Vec<&str> = without_footer
        .lines()
        .filter(|line| {
            let t = line.trim();
            !(t == "🤖" || url_line.is_match(t) || coauthor.is_match(t))
        })
        .collect();

    // Drop a trailing horizontal rule (the `---` that introduced the footer)
    // along with any trailing blank lines it leaves behind.
    let mut out = kept.join("\n");
    out = out.trim_end().to_string();
    if let Some(rest) = out.strip_suffix("---")
        && (rest.is_empty() || rest.ends_with('\n'))
    {
        out = rest.trim_end().to_string();
    }

    blanks.replace_all(&out, "\n\n").trim().to_string()
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

    // Interactive force-directed graph view (Obsidian-style). Mirrors the set
    // of node pages present on disk so links always resolve.
    write_file(
        &opts.out_dir.join("graph.md"),
        &render_graph_page(nodes, wiki_dir),
    )?;
    pages_written += 1;

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

// --- Interactive graph view -------------------------------------------------

/// CDN-hosted force-directed graph renderer (2D canvas, pan/zoom/drag). Pinned
/// to the 1.x line. Loaded at view time by the visitor's browser, so the build
/// itself needs no network access.
const FORCE_GRAPH_SRC: &str = "https://cdn.jsdelivr.net/npm/force-graph@1/dist/force-graph.min.js";

#[derive(Serialize)]
struct GraphNode {
    id: String,
    label: String,
    area: String,
    color: String,
    /// Relative node size — degree + 1 so isolated nodes are still visible.
    val: usize,
    /// Page to open on click (relative to the graph page, same folder).
    url: String,
    /// De-emphasized in the view (rendered faded + smaller): peripheral history
    /// now that dex is a pure-Rust binary — the legacy Python bindings, and
    /// documentation/CI plumbing that isn't an architectural decision.
    muted: bool,
}

#[derive(Serialize)]
struct GraphLink {
    source: String,
    target: String,
    kind: String,
}

#[derive(Serialize)]
struct GraphLegend {
    title: String,
    color: String,
}

#[derive(Serialize)]
struct GraphData {
    nodes: Vec<GraphNode>,
    links: Vec<GraphLink>,
    areas: Vec<GraphLegend>,
}

/// Build the node/edge payload for the interactive graph, restricted to nodes
/// whose page exists on disk (so every click target resolves).
fn build_graph_data(nodes: &[Node], wiki_dir: &Path) -> GraphData {
    let present: std::collections::HashSet<&str> = nodes
        .iter()
        .filter(|n| wiki_dir.join(format!("{}.md", n.stem)).exists())
        .map(|n| n.stem.as_str())
        .collect();

    // Links first (both endpoints must be present), then degree from links.
    let mut links = Vec::new();
    let mut degree: BTreeMap<String, usize> = BTreeMap::new();
    for node in nodes {
        if !present.contains(node.stem.as_str()) {
            continue;
        }
        for e in &node.edges {
            if node.stem == e.target_stem || !present.contains(e.target_stem.as_str()) {
                continue;
            }
            *degree.entry(node.stem.clone()).or_insert(0) += 1;
            *degree.entry(e.target_stem.clone()).or_insert(0) += 1;
            links.push(GraphLink {
                source: node.stem.clone(),
                target: e.target_stem.clone(),
                kind: e.kind.wikilink_label().to_string(),
            });
        }
    }

    let graph_nodes: Vec<GraphNode> = nodes
        .iter()
        .filter(|n| present.contains(n.stem.as_str()))
        .map(|n| GraphNode {
            id: n.stem.clone(),
            label: format!(
                "{} {} — {} · {}",
                n.commit.short_sha,
                n.class.label(),
                n.commit.subject,
                n.area.title()
            ),
            area: n.area.title().to_string(),
            color: n.area.color().to_string(),
            val: degree.get(&n.stem).copied().unwrap_or(0) + 1,
            url: format!("{}.html", n.stem),
            muted: is_muted(n),
        })
        .collect();

    // Legend: only areas that actually have nodes, in canonical order.
    let areas: Vec<GraphLegend> = FunctionalArea::ALL
        .iter()
        .filter(|a| {
            nodes
                .iter()
                .any(|n| n.area == **a && present.contains(n.stem.as_str()))
        })
        .map(|a| GraphLegend {
            title: a.title().to_string(),
            color: a.color().to_string(),
        })
        .collect();

    GraphData {
        nodes: graph_nodes,
        links,
        areas,
    }
}

/// Whether a node should be de-emphasized ("muted") in the graph view. dex is a
/// pure-Rust binary now, so two kinds of history are real but no longer central
/// and read better faded into the background:
///
/// - the **legacy Python** layer (the `dex-py` PyO3 bindings and any pre-port
///   Python sources), and
/// - **documentation / CI plumbing** that isn't an architectural decision
///   (release fixes, docs-site tweaks) — the `[Decision]` design docs stay
///   full-strength so the product's actual pivots remain prominent.
///
/// A node carrying real Rust source (`crates/**/src/*.rs`) is never muted on the
/// docs/CI rule: that catches core-code features whose file-majority happened to
/// route them into the docs area (e.g. a feature that also commits generated
/// `.context/` pages), keeping them bright where they belong.
fn is_muted(n: &Node) -> bool {
    is_python_legacy(&n.signal_files)
        || (n.area == FunctionalArea::DocsCiRelease
            && n.class != NodeClass::Decision
            && !touches_rust_source(&n.signal_files))
}

/// Whether any changed file is hand-written Rust crate source.
fn touches_rust_source(signal_files: &[String]) -> bool {
    signal_files.iter().any(|f| {
        let p = f.replace('\\', "/");
        p.starts_with("crates/") && p.contains("/src/") && p.ends_with(".rs")
    })
}

/// True when a path belongs to dex's own Python layer (not a Python *template*,
/// which is project content rendered for users).
fn is_python_file(path: &str) -> bool {
    let p = path.replace('\\', "/");
    if p.starts_with("templates/") {
        return false;
    }
    p.starts_with("crates/dex-py/")
        || p.starts_with("python/")
        || p == "pyproject.toml"
        || p.ends_with("/pyproject.toml")
        || p.ends_with(".py")
        || p.ends_with(".pyi")
}

/// A commit is "legacy Python" when its tracked changes are *predominantly* the
/// Python layer. The majority test keeps the pivotal "port to pure Rust" commit
/// bright (it adds far more Rust than the Python it removes) while fading commits
/// that only maintained the bindings.
fn is_python_legacy(signal_files: &[String]) -> bool {
    if signal_files.is_empty() {
        return false;
    }
    let py = signal_files.iter().filter(|f| is_python_file(f)).count();
    py * 2 > signal_files.len()
}

/// Render the `graph.md` page: a self-contained interactive force-directed
/// graph with the data embedded inline as JSON (no fetch, no path juggling).
fn render_graph_page(nodes: &[Node], wiki_dir: &Path) -> String {
    let data = build_graph_data(nodes, wiki_dir);
    // Serialization can't fail for these plain types; fall back to an empty
    // graph rather than panicking if it somehow does.
    let json = serde_json::to_string(&data)
        .unwrap_or_else(|_| "{\"nodes\":[],\"links\":[],\"areas\":[]}".to_string());

    format!(
        r#"# Project Memory — Graph

An interactive map of every significant commit, colored by functional area.
**Drag** to pan, **scroll** to zoom. **Tap a node** to see its details; tap it
**again** (or the **Open** link) to jump to that commit's page.

<div id="pm-graph" style="width:100%;height:70vh;border:1px solid rgba(128,128,128,0.4);border-radius:6px;overflow:hidden"></div>
<div id="pm-graph-info" style="margin-top:.6rem;font-size:.9em;min-height:1.6em;line-height:1.5"></div>
<div id="pm-graph-legend" style="margin-top:.5rem;font-size:.85em;line-height:1.9"></div>
<p style="margin-top:.4rem;font-size:.8em;opacity:.65">Faded nodes are de-emphasized history — the legacy Python bindings and documentation/CI plumbing — kept for the record but no longer central now that dex is a pure-Rust binary.</p>

<script type="application/json" id="pm-graph-data">
{json}
</script>
<script src="{src}"></script>
<script>
(function () {{
  var el = document.getElementById('pm-graph');
  var raw = document.getElementById('pm-graph-data');
  if (!el || !raw) return;
  var data;
  try {{ data = JSON.parse(raw.textContent); }} catch (e) {{ return; }}

  if (typeof ForceGraph === 'undefined') {{
    el.innerHTML = '<p style="padding:1rem">The interactive graph needs JavaScript and ' +
      'network access to load. Browse the <a href="INDEX.html">Knowledge Map</a> instead.</p>';
    return;
  }}

  function esc(s) {{
    return String(s).replace(/[&<>"]/g, function (c) {{
      return {{ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }}[c];
    }});
  }}

  // Render a `#rrggbb` area color at low opacity so muted nodes recede.
  function fade(hex) {{
    var m = /^#?([0-9a-f]{{2}})([0-9a-f]{{2}})([0-9a-f]{{2}})$/i.exec(hex);
    if (!m) return hex;
    return 'rgba(' + parseInt(m[1], 16) + ',' + parseInt(m[2], 16) + ',' +
      parseInt(m[3], 16) + ',0.22)';
  }}

  var info = document.getElementById('pm-graph-info');
  var legend = document.getElementById('pm-graph-legend');
  if (legend && data.areas) {{
    legend.innerHTML = data.areas.map(function (a) {{
      return '<span style="display:inline-block;margin-right:1rem;white-space:nowrap">' +
        '<span style="display:inline-block;width:.7em;height:.7em;border-radius:50%;vertical-align:middle;' +
        'background:' + a.color + ';margin-right:.35em"></span>' + a.title + '</span>';
    }}).join('');
  }}

  var selected = null;
  function showInfo(n) {{
    if (!info) return;
    info.innerHTML =
      '<span style="display:inline-block;width:.7em;height:.7em;border-radius:50%;vertical-align:middle;' +
      'background:' + n.color + ';margin-right:.4em"></span>' +
      '<strong>' + esc(n.label) + '</strong>' +
      (n.url ? ' &nbsp;<a href="' + esc(n.url) + '">Open →</a>' : '');
  }}
  function clearInfo() {{ selected = null; if (info) info.textContent = ''; }}

  var graph = ForceGraph()(el)
    .graphData({{ nodes: data.nodes, links: data.links }})
    .nodeId('id')
    .nodeLabel('label')
    .nodeColor(function (n) {{ return n.muted ? fade(n.color) : n.color; }})
    .nodeVal(function (n) {{ return n.muted ? Math.max(1, n.val * 0.5) : n.val; }})
    .nodeRelSize(4)
    .linkColor(function () {{ return 'rgba(128,128,128,0.35)'; }})
    .linkDirectionalArrowLength(2.5)
    .linkDirectionalArrowRelPos(1)
    .onNodeClick(function (n) {{
      // First tap selects + reveals details; a second tap on the same node
      // (or the Open link) navigates. Touch-friendly — no hover needed.
      if (selected === n) {{ if (n.url) window.location.href = n.url; return; }}
      selected = n;
      showInfo(n);
      graph.centerAt(n.x, n.y, 400);
      graph.zoom(Math.max(graph.zoom(), 3), 400);
    }})
    .onBackgroundClick(clearInfo)
    .width(el.clientWidth)
    .height(el.clientHeight);

  graph.onEngineStop(function () {{ graph.zoomToFit(400, 40); }});
  window.addEventListener('resize', function () {{
    graph.width(el.clientWidth).height(el.clientHeight);
  }});
}}());
</script>
"#,
        json = json,
        src = FORCE_GRAPH_SRC,
    )
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
    s.push_str(&format!("- [Graph view]({dir_name}/graph.md)\n"));
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
    fn graph_page_embeds_present_nodes_and_links() {
        use super::super::build_nodes;
        use crate::context_graph::git::RawCommit;

        // Two commits touching a shared file → a co-occurrence edge between them.
        let commits = vec![
            RawCommit {
                sha: "1111111111111111111111111111111111111111".into(),
                short_sha: "1111111".into(),
                author: "A".into(),
                date: "2024-01-01".into(),
                subject: "feat: add engine".into(),
                body: String::new(),
                files: vec!["crates/dex-core/src/engine.rs".into()],
            },
            RawCommit {
                sha: "2222222222222222222222222222222222222222".into(),
                short_sha: "2222222".into(),
                author: "A".into(),
                date: "2024-01-02".into(),
                subject: "fix: engine bug".into(),
                body: String::new(),
                files: vec!["crates/dex-core/src/engine.rs".into()],
            },
        ];
        let nodes = build_nodes(&commits);

        // Pretend both node pages exist on disk so they're "present".
        let dir = tempfile::tempdir().unwrap();
        for n in &nodes {
            std::fs::write(dir.path().join(format!("{}.md", n.stem)), "x").unwrap();
        }

        let data = build_graph_data(&nodes, dir.path());
        assert_eq!(data.nodes.len(), 2, "both present nodes are included");
        assert!(!data.links.is_empty(), "shared file yields an edge");
        assert!(!data.areas.is_empty(), "legend lists active areas");
        // Every link endpoint must be a real node id.
        let ids: std::collections::HashSet<&str> =
            data.nodes.iter().map(|n| n.id.as_str()).collect();
        for l in &data.links {
            assert!(ids.contains(l.source.as_str()));
            assert!(ids.contains(l.target.as_str()));
        }

        let page = render_graph_page(&nodes, dir.path());
        assert!(page.contains("id=\"pm-graph\""));
        assert!(page.contains("force-graph"));
        // Click targets point at the rendered .html pages.
        assert!(page.contains(&format!("{}.html", nodes[0].stem)));
    }

    #[test]
    fn muting_fades_python_and_docs_churn_only() {
        // Legacy Python (majority .py / dex-py) → muted.
        assert!(is_python_legacy(&[
            "crates/dex-py/src/lib.rs".into(),
            "python/dex/cli.py".into(),
            "pyproject.toml".into(),
        ]));
        // A Rust-majority commit that merely touches a .py is NOT legacy Python
        // (keeps the "port to pure Rust" pivot bright).
        assert!(!is_python_legacy(&[
            "crates/dex-core/src/lib.rs".into(),
            "crates/dex-cli/src/main.rs".into(),
            "old/cli.py".into(),
        ]));
        // Python *templates* are user content, not dex's Python layer.
        assert!(!is_python_legacy(&[
            "templates/python-package/main.py".into()
        ]));

        use super::super::build_nodes;
        use crate::context_graph::git::RawCommit;
        let raw = |sha: &str, subject: &str, files: &[&str]| RawCommit {
            sha: format!("{sha}0000000000000000000000000000000000"),
            short_sha: sha.into(),
            author: "A".into(),
            date: "2026-01-01".into(),
            subject: subject.into(),
            body: String::new(),
            files: files.iter().map(|s| s.to_string()).collect(),
        };
        let nodes = build_nodes(&[
            raw(
                "aaaaaaa",
                "fix(release): tweak workflow",
                &[".github/workflows/release.yml"],
            ),
            raw("bbbbbbb", "docs: add PRD for X", &["docs/prd-x.md"]),
            raw(
                "ccccccc",
                "feat(core): add scaffold",
                &["crates/dex-core/src/scaffold.rs"],
            ),
            // A core-code feature whose docs/.context file-majority routes it
            // into the docs area — must stay bright (carries Rust source).
            raw(
                "ddddddd",
                "feat(context): engine",
                &[
                    "crates/dex-core/src/context_graph/mod.rs",
                    "docs/SPEC.md",
                    ".github/workflows/pages.yml",
                    ".context/wiki/INDEX.md",
                ],
            ),
        ]);
        let by = |sub: &str| nodes.iter().find(|n| n.commit.subject == sub).unwrap();

        // Docs/CI plumbing that isn't a decision → muted.
        assert!(is_muted(by("fix(release): tweak workflow")));
        // A design-doc decision stays bright.
        assert!(!is_muted(by("docs: add PRD for X")));
        // Core Rust code stays bright.
        assert!(!is_muted(by("feat(core): add scaffold")));
        // Misrouted core-code feature stays bright (carries Rust source).
        assert_eq!(
            by("feat(context): engine").area,
            FunctionalArea::DocsCiRelease
        );
        assert!(!is_muted(by("feat(context): engine")));
    }

    #[test]
    fn graph_data_excludes_absent_nodes() {
        use super::super::build_nodes;
        use crate::context_graph::git::RawCommit;

        let commits = vec![RawCommit {
            sha: "3333333333333333333333333333333333333333".into(),
            short_sha: "3333333".into(),
            author: "A".into(),
            date: "2024-01-03".into(),
            subject: "feat: lonely".into(),
            body: String::new(),
            files: vec!["crates/dex-core/src/x.rs".into()],
        }];
        let nodes = build_nodes(&commits);

        // Empty dir: no node page exists → nothing is present.
        let dir = tempfile::tempdir().unwrap();
        let data = build_graph_data(&nodes, dir.path());
        assert!(data.nodes.is_empty());
        assert!(data.links.is_empty());
    }

    #[test]
    fn clean_body_strips_ai_attribution_footers() {
        // Real-world footer: bare URL, wrapped "Generated by" link, and a
        // Co-authored-by trailer, all introduced by a `---` rule.
        let body = "Real change description.\n\nMore detail here.\n\n\
https://claude.ai/code/session_014jkL5QkJ44uE6seaXSftQt\n\n---\n\
_Generated by [Claude\nCode](https://claude.ai/code/session_014jkL5QkJ44uE6seaXSftQt)_\n\n\
Co-authored-by: Claude <noreply@anthropic.com>";
        let cleaned = clean_body(body);
        assert!(cleaned.starts_with("Real change description."));
        assert!(cleaned.contains("More detail here."));
        assert!(!cleaned.contains("claude.ai"), "URL removed");
        assert!(
            !cleaned.to_lowercase().contains("generated by"),
            "footer removed"
        );
        assert!(
            !cleaned.to_lowercase().contains("co-authored-by"),
            "trailer removed"
        );
        assert!(!cleaned.contains("---"), "orphaned rule removed");
        assert!(!cleaned.ends_with('\n'), "no trailing whitespace");
    }

    #[test]
    fn clean_body_strips_robot_generated_with() {
        let body = "Add thing.\n\n🤖 Generated with [Claude Code](https://claude.com/claude-code)\n\n\
Co-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>";
        let cleaned = clean_body(body);
        assert_eq!(cleaned, "Add thing.");
    }

    #[test]
    fn clean_body_keeps_human_content_and_coauthors() {
        // A non-Claude co-author and a real issue ref must survive.
        let body = "Fix the parser (#42).\n\nCo-authored-by: Jane Dev <jane@example.com>";
        let cleaned = clean_body(body);
        assert!(cleaned.contains("#42"));
        assert!(
            cleaned.contains("Jane Dev"),
            "human co-authors are preserved"
        );
    }

    #[test]
    fn clean_body_leaves_plain_bodies_untouched() {
        assert_eq!(
            clean_body("Just a normal message."),
            "Just a normal message."
        );
        assert_eq!(clean_body(""), "");
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
