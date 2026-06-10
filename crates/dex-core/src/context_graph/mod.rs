//! Project-memory engine — build a *Project Evolution Knowledge Graph* under
//! `.context/wiki/`.
//!
//! Produces one Markdown node per significant commit (`<sha>-<slug>.md`),
//! stitched together with Obsidian-style `[[wikilink]]` edges, plus an
//! `INDEX.md` "city map" grouped by functional area. The output renders as a
//! live graph in Obsidian / Logseq / VS Code (Foam) and gives both humans and
//! AI agents a navigable record of *why* the codebase looks the way it does.
//!
//! This is the deterministic, Rust-native counterpart to the
//! `project-memory-engine` skill: it does the mechanical work (git parsing,
//! conventional-commit classification, co-change analysis, indexing) with no
//! AI in the loop. The skill layers semantic enrichment on top.
//!
//! Tuned for dex's repository structure (see [`FunctionalArea`]).

pub mod git;
pub mod render;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use regex::Regex;

use crate::error::DexError;

pub use git::RawCommit;

/// Classification of a commit's role in the project's evolution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeClass {
    /// Architectural pivot.
    Decision,
    /// Major feature / capability addition.
    Evolution,
    /// Bug fix, hardening, resilience.
    Stability,
    /// Environment / config / packaging.
    Dependency,
}

impl NodeClass {
    /// Label as it appears in node titles and the index, e.g. `[Decision]`.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            NodeClass::Decision => "[Decision]",
            NodeClass::Evolution => "[Evolution]",
            NodeClass::Stability => "[Stability]",
            NodeClass::Dependency => "[Dependency]",
        }
    }

    /// Obsidian tag form, e.g. `#decision`.
    #[must_use]
    pub fn tag(self) -> &'static str {
        match self {
            NodeClass::Decision => "#decision",
            NodeClass::Evolution => "#evolution",
            NodeClass::Stability => "#stability",
            NodeClass::Dependency => "#dependency",
        }
    }
}

/// Functional areas of the dex codebase. Order is the reading order used in the
/// index; it doubles as the tie-breaker when a commit spans several areas
/// (lower = more foundational wins).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum FunctionalArea {
    Foundation,
    TemplateEngine,
    Scaffolding,
    Cli,
    SkillsTraits,
    McpAi,
    Templates,
    DocsCiRelease,
}

impl FunctionalArea {
    /// All areas in index/reading order.
    pub const ALL: [FunctionalArea; 8] = [
        FunctionalArea::Foundation,
        FunctionalArea::TemplateEngine,
        FunctionalArea::Scaffolding,
        FunctionalArea::Cli,
        FunctionalArea::SkillsTraits,
        FunctionalArea::McpAi,
        FunctionalArea::Templates,
        FunctionalArea::DocsCiRelease,
    ];

    /// Human-readable title used as the index section heading.
    #[must_use]
    pub fn title(self) -> &'static str {
        match self {
            FunctionalArea::Foundation => "Foundation & Architecture",
            FunctionalArea::TemplateEngine => "Template Engine & Rendering",
            FunctionalArea::Scaffolding => "Scaffolding & Project Generation",
            FunctionalArea::Cli => "CLI & Interfaces",
            FunctionalArea::SkillsTraits => "Skills, Traits & Extensibility",
            FunctionalArea::McpAi => "MCP & AI Integration",
            FunctionalArea::Templates => "Templates & Built-in Content",
            FunctionalArea::DocsCiRelease => "Docs, CI & Release",
        }
    }

    /// Distinct color (hex) for this area, used by the interactive graph view.
    /// Chosen to stay legible on both the light and dark mdBook themes.
    #[must_use]
    pub fn color(self) -> &'static str {
        match self {
            FunctionalArea::Foundation => "#e6194b",
            FunctionalArea::TemplateEngine => "#3cb44b",
            FunctionalArea::Scaffolding => "#4363d8",
            FunctionalArea::Cli => "#f58231",
            FunctionalArea::SkillsTraits => "#911eb4",
            FunctionalArea::McpAi => "#22b8cf",
            FunctionalArea::Templates => "#f032e6",
            FunctionalArea::DocsCiRelease => "#9a6324",
        }
    }

    /// Map a single repo-relative path to its functional area.
    #[must_use]
    pub fn of_path(path: &str) -> FunctionalArea {
        let p = path.replace('\\', "/");
        if p.starts_with("crates/dex-core/src/template") {
            FunctionalArea::TemplateEngine
        } else if p.starts_with("crates/dex-core/src/scaffold")
            || p.starts_with("crates/dex-core/src/apply_trait")
            || p.starts_with("crates/dex-core/src/context_map")
            || p.starts_with("crates/dex-core/src/context_graph")
        {
            FunctionalArea::Scaffolding
        } else if p.starts_with("crates/dex-core/src/mcp") {
            FunctionalArea::McpAi
        } else if p.starts_with("crates/dex-core/src/skills")
            || p.starts_with("crates/dex-core/src/traits")
            || p.starts_with("skills/")
            || p.starts_with("traits/")
        {
            FunctionalArea::SkillsTraits
        } else if p.starts_with("crates/dex-cli")
            || p.starts_with("crates/dex-py")
            || p.starts_with("webapp/")
        {
            FunctionalArea::Cli
        } else if p.starts_with("templates/") {
            FunctionalArea::Templates
        } else if p.starts_with("docs/")
            || p.starts_with(".github/")
            || p.starts_with("scripts/")
            || p.starts_with(".context/")
            || p == "install.sh"
            || p == "book.toml"
            || p == "cliff.toml"
        {
            FunctionalArea::DocsCiRelease
        } else {
            // Cargo.toml, Cargo.lock, lib.rs, config.rs, error.rs, README, etc.
            FunctionalArea::Foundation
        }
    }
}

/// The kind of relationship one node has to another.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EdgeKind {
    /// A feature/decision builds on a prior decision.
    InfluencedBy,
    /// A fix alters a design pattern introduced elsewhere.
    ModifiedBy,
    /// A code change realises a design-doc node.
    ImplementedIn,
    /// Files that consistently change together (coupling).
    CoOccurrence,
}

impl EdgeKind {
    #[must_use]
    pub fn wikilink_label(self) -> &'static str {
        match self {
            EdgeKind::InfluencedBy => "influenced-by",
            EdgeKind::ModifiedBy => "modified-by",
            EdgeKind::ImplementedIn => "implemented-in",
            EdgeKind::CoOccurrence => "co-occurrence",
        }
    }
}

/// A directed edge from one node to another node (by filename stem).
#[derive(Debug, Clone)]
pub struct Edge {
    pub kind: EdgeKind,
    /// Target node filename stem (`<short_sha>-<slug>`), used in `[[wikilinks]]`.
    pub target_stem: String,
    /// Optional short annotation (e.g. "2 shared files").
    pub note: Option<String>,
}

/// A fully-resolved knowledge-graph node for one commit.
#[derive(Debug, Clone)]
pub struct Node {
    pub commit: RawCommit,
    pub class: NodeClass,
    pub area: FunctionalArea,
    pub slug: String,
    /// Filename stem: `<short_sha>-<slug>`.
    pub stem: String,
    /// Changed files with vendored/lockfile noise removed.
    pub signal_files: Vec<String>,
    /// Issue / PR references parsed from the message (e.g. `#42`, `ABC-7`).
    pub issue_refs: Vec<String>,
    pub edges: Vec<Edge>,
}

/// Options controlling a sync run.
#[derive(Debug, Clone, Default)]
pub struct SyncOptions {
    /// Wipe and regenerate every node, ignoring what's already on disk.
    pub rebuild: bool,
    /// Cap history to the most recent `N` commits (first-run convenience).
    pub limit: Option<usize>,
    /// Keep every commit. By default the [Dependency] class (releases, version
    /// bumps, CI tweaks, docs plumbing) is filtered out so the graph is code
    /// changes and major decisions only. Set to include the mechanical churn too.
    pub include_all: bool,
}

/// Outcome of a sync run, for the CLI to render.
#[derive(Debug, Clone)]
pub struct SyncReport {
    pub wiki_dir: PathBuf,
    pub index_path: PathBuf,
    pub manual_path: PathBuf,
    /// Total nodes in the graph after this run.
    pub nodes_total: usize,
    /// Node files newly written this run (skipped if already present).
    pub nodes_written: usize,
    pub manual_written: bool,
    /// Node count per area, in [`FunctionalArea::ALL`] order.
    pub area_counts: Vec<(FunctionalArea, usize)>,
}

/// Build or refresh the project-memory graph rooted at `root`.
///
/// Writes to `<root>/.context/wiki/`. Incremental by default: existing node
/// files are left untouched and only missing commits get new nodes; `INDEX.md`
/// is always rewritten so it reflects the full graph.
pub fn sync(root: &Path, opts: &SyncOptions) -> Result<SyncReport, DexError> {
    git::ensure_repo(root)?;
    let commits = git::log(root, opts.limit)?;
    let commits = filter_commits(commits, opts.include_all);
    let nodes = build_nodes(&commits);

    let wiki_dir = root.join(".context").join("wiki");
    let manual_path = root.join(".context").join("USER_MANUAL.md");
    let index_path = wiki_dir.join("INDEX.md");

    render::write_graph(&wiki_dir, &manual_path, &index_path, &nodes, opts)
}

/// Options controlling an mdBook export.
#[derive(Debug, Clone)]
pub struct ExportOptions {
    /// Directory to write mdBook-ready pages into (e.g. `docs/wiki`).
    pub out_dir: PathBuf,
    /// `SUMMARY.md` to inject a navigation section into (between markers). When
    /// `None`, no SUMMARY is touched.
    pub summary_path: Option<PathBuf>,
    /// Keep every commit. Mirrors [`SyncOptions::include_all`]; defaults to the
    /// same [Dependency]-class filter so the published graph matches the synced one.
    pub include_all: bool,
}

/// Outcome of an export run.
#[derive(Debug, Clone)]
pub struct ExportReport {
    pub out_dir: PathBuf,
    pub pages_written: usize,
    pub summary_updated: bool,
}

/// Render the committed `.context/wiki/` into mdBook-compatible pages.
///
/// Copies the index, manual, and node files into `opts.out_dir`, rewriting
/// `[[wikilinks]]` into relative mdBook links, and optionally injects a
/// navigation section into a `SUMMARY.md`. The graph in `.context/wiki/`
/// remains the editable source of truth; this is a derived view for the website.
pub fn export(root: &Path, opts: &ExportOptions) -> Result<ExportReport, DexError> {
    git::ensure_repo(root)?;
    let commits = git::log(root, None)?;
    let commits = filter_commits(commits, opts.include_all);
    let nodes = build_nodes(&commits);

    let wiki_dir = root.join(".context").join("wiki");
    let manual_path = root.join(".context").join("USER_MANUAL.md");

    render::write_export(&wiki_dir, &manual_path, &nodes, opts)
}

/// Turn raw commits into classified, edge-stitched nodes.
///
/// Exposed for testing; `sync` is the normal entry point.
#[must_use]
pub fn build_nodes(commits: &[RawCommit]) -> Vec<Node> {
    // First pass: classify and assign areas. Slugs must be unique.
    let mut used_stems: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut nodes: Vec<Node> = Vec::with_capacity(commits.len());

    for c in commits {
        let signal_files: Vec<String> = c.files.iter().filter(|f| !is_noise(f)).cloned().collect();
        let class = classify(c, &signal_files);
        // Prefer the commit scope for area (e.g. `feat(mcp)` → MCP & AI); fall
        // back to file-majority when there's no scope or it isn't an area.
        let area = scope_of(&c.subject)
            .as_deref()
            .and_then(area_from_scope)
            .unwrap_or_else(|| area_of(&signal_files));
        let mut stem = format!("{}-{}", c.short_sha, slugify(&c.subject));
        // Guard against the (rare) duplicate stem.
        while !used_stems.insert(stem.clone()) {
            stem.push('_');
        }
        let slug = stem
            .strip_prefix(&format!("{}-", c.short_sha))
            .unwrap_or(&stem)
            .to_string();
        let issue_refs = parse_issue_refs(&format!("{} {}", c.subject, c.body));

        nodes.push(Node {
            commit: c.clone(),
            class,
            area,
            slug,
            stem,
            signal_files,
            issue_refs,
            edges: Vec::new(),
        });
    }

    stitch_edges(&mut nodes);
    nodes
}

// --- Classification ---------------------------------------------------------

fn conventional_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"^(?P<type>[a-zA-Z]+)(?:\((?P<scope>[^)]*)\))?(?P<bang>!)?:").unwrap()
    })
}

/// The conventional-commit scope, lowercased (e.g. `mcp` from `feat(mcp): …`).
fn scope_of(subject: &str) -> Option<String> {
    conventional_re()
        .captures(subject)
        .and_then(|caps| caps.name("scope"))
        .map(|m| m.as_str().to_ascii_lowercase())
}

/// Map a conventional-commit scope to a functional area when it names one
/// directly — a far stronger signal than file-majority for scoped commits.
fn area_from_scope(scope: &str) -> Option<FunctionalArea> {
    match scope {
        "mcp" => Some(FunctionalArea::McpAi),
        "cli" => Some(FunctionalArea::Cli),
        "scaffold" | "context-map" | "context_map" | "trait" => Some(FunctionalArea::Scaffolding),
        "template" | "engine" | "render" => Some(FunctionalArea::TemplateEngine),
        "templates" | "agent" => Some(FunctionalArea::Templates),
        "skills" | "traits" => Some(FunctionalArea::SkillsTraits),
        "config" | "core" => Some(FunctionalArea::Foundation),
        "devcontainer" | "ci" | "release" | "deps" | "build" | "docs" => {
            Some(FunctionalArea::DocsCiRelease)
        }
        _ => None,
    }
}

/// Classify a commit using its conventional-commit prefix, breaking-change
/// markers, and the files it touched.
fn classify(c: &RawCommit, signal_files: &[String]) -> NodeClass {
    let breaking = conventional_re()
        .captures(&c.subject)
        .and_then(|caps| caps.name("bang"))
        .is_some()
        || c.body.contains("BREAKING CHANGE");
    if breaking {
        return NodeClass::Decision;
    }

    let ty = conventional_re()
        .captures(&c.subject)
        .and_then(|caps| caps.name("type"))
        .map(|m| m.as_str().to_ascii_lowercase());

    match ty.as_deref() {
        Some("feat") => NodeClass::Evolution,
        Some("fix" | "perf" | "revert" | "test") => NodeClass::Stability,
        Some("refactor") => NodeClass::Decision,
        Some("build" | "ci" | "chore" | "deps" | "release" | "bump" | "style") => {
            NodeClass::Dependency
        }
        Some("docs") => {
            if signal_files.iter().any(|f| is_design_doc(f)) {
                NodeClass::Decision
            } else {
                NodeClass::Dependency
            }
        }
        // No `type:` prefix. Catch common non-colon squash titles (e.g.
        // "Chore/release v0.1.4", "Feat/user config") by their first word,
        // then fall back to inference from the files touched.
        _ => match loose_keyword(&c.subject) {
            Some(class) => class,
            None => {
                if signal_files.iter().any(|f| is_design_doc(f)) {
                    NodeClass::Decision
                } else if !signal_files.is_empty()
                    && signal_files.iter().all(|f| f.starts_with("docs/"))
                {
                    NodeClass::Dependency
                } else {
                    NodeClass::Evolution
                }
            }
        },
    }
}

/// Best-effort class from a subject's leading word when there's no `type:`
/// prefix (handles slash-style squash-merge titles).
fn loose_keyword(subject: &str) -> Option<NodeClass> {
    let first: String = subject
        .chars()
        .take_while(|c| c.is_ascii_alphabetic())
        .collect::<String>()
        .to_ascii_lowercase();
    match first.as_str() {
        "feat" | "feature" => Some(NodeClass::Evolution),
        "fix" | "bugfix" | "hotfix" => Some(NodeClass::Stability),
        "chore" | "release" | "bump" | "ci" | "build" | "deps" => Some(NodeClass::Dependency),
        "refactor" => Some(NodeClass::Decision),
        _ => None,
    }
}

fn area_of(signal_files: &[String]) -> FunctionalArea {
    if signal_files.is_empty() {
        return FunctionalArea::Foundation;
    }
    // Count files per area, then pick the busiest (ties → most foundational).
    let mut counts: BTreeMap<FunctionalArea, usize> = BTreeMap::new();
    for f in signal_files {
        *counts.entry(FunctionalArea::of_path(f)).or_insert(0) += 1;
    }
    counts
        .into_iter()
        .max_by(|a, b| a.1.cmp(&b.1).then(b.0.cmp(&a.0)))
        .map(|(area, _)| area)
        .unwrap_or(FunctionalArea::Foundation)
}

// --- Edge stitching ---------------------------------------------------------

fn stitch_edges(nodes: &mut [Node]) {
    // File frequency, to suppress hub files (e.g. Cargo.toml, lib.rs) from
    // co-occurrence — they couple to everything and add noise without signal.
    let mut freq: BTreeMap<String, usize> = BTreeMap::new();
    for n in nodes.iter() {
        for f in &n.signal_files {
            *freq.entry(f.clone()).or_insert(0) += 1;
        }
    }
    let hub_threshold = std::cmp::max(4, nodes.len() * 2 / 5); // 40%, min 4
    let is_hub = |f: &str| freq.get(f).copied().unwrap_or(0) > hub_threshold;

    for i in 0..nodes.len() {
        let mut edges: Vec<Edge> = Vec::new();
        let cur_files: Vec<&String> = nodes[i]
            .signal_files
            .iter()
            .filter(|f| !is_hub(f))
            .collect();
        let cur_area = nodes[i].area;
        let cur_class = nodes[i].class;

        // co-occurrence: prior nodes sharing the most non-hub files (top 3).
        let mut shared: Vec<(usize, usize)> = Vec::new(); // (prior index, shared count)
        for (j, prior) in nodes[..i].iter().enumerate() {
            let count = prior
                .signal_files
                .iter()
                .filter(|f| !is_hub(f) && cur_files.contains(f))
                .count();
            if count > 0 {
                shared.push((j, count));
            }
        }
        // Most shared first, then most recent.
        shared.sort_by(|a, b| b.1.cmp(&a.1).then(b.0.cmp(&a.0)));
        for (j, count) in shared.iter().take(3) {
            edges.push(Edge {
                kind: EdgeKind::CoOccurrence,
                target_stem: nodes[*j].stem.clone(),
                note: Some(format!(
                    "{count} shared file{}",
                    if *count == 1 { "" } else { "s" }
                )),
            });
        }

        // influenced-by: most recent prior Decision in the same area.
        if matches!(cur_class, NodeClass::Evolution | NodeClass::Decision)
            && let Some(j) = last_prior(nodes, i, |n| {
                n.class == NodeClass::Decision && n.area == cur_area
            })
        {
            push_unique(
                &mut edges,
                EdgeKind::InfluencedBy,
                &nodes[j].stem,
                Some(nodes[j].area.title().to_string()),
            );
        }

        // modified-by: a fix alters a prior Decision/Evolution it shares files with.
        if cur_class == NodeClass::Stability {
            let target = last_prior(nodes, i, |n| {
                matches!(n.class, NodeClass::Decision | NodeClass::Evolution)
                    && n.signal_files
                        .iter()
                        .any(|f| !is_hub(f) && cur_files.contains(&f))
            })
            .or_else(|| {
                last_prior(nodes, i, |n| {
                    matches!(n.class, NodeClass::Decision | NodeClass::Evolution)
                        && n.area == cur_area
                })
            });
            if let Some(j) = target {
                push_unique(&mut edges, EdgeKind::ModifiedBy, &nodes[j].stem, None);
            }
        }

        // implemented-in: a code change realises a prior design-doc node.
        let touches_design = nodes[i].signal_files.iter().any(|f| is_design_doc(f));
        let touches_code = nodes[i]
            .signal_files
            .iter()
            .any(|f| !f.starts_with("docs/") && !is_design_doc(f));
        if touches_code
            && !touches_design
            && let Some(j) = last_prior(nodes, i, |n| {
                n.signal_files.iter().any(|f| is_design_doc(f))
            })
        {
            push_unique(&mut edges, EdgeKind::ImplementedIn, &nodes[j].stem, None);
        }

        nodes[i].edges = edges;
    }
}

/// Index of the most recent prior node (searching backwards from `i`) matching
/// `pred`.
fn last_prior(nodes: &[Node], i: usize, pred: impl Fn(&Node) -> bool) -> Option<usize> {
    (0..i).rev().find(|&j| pred(&nodes[j]))
}

fn push_unique(edges: &mut Vec<Edge>, kind: EdgeKind, target: &str, note: Option<String>) {
    if !edges
        .iter()
        .any(|e| e.kind == kind && e.target_stem == target)
    {
        edges.push(Edge {
            kind,
            target_stem: target.to_string(),
            note,
        });
    }
}

// --- Helpers ----------------------------------------------------------------

fn parse_issue_refs(text: &str) -> Vec<String> {
    static RE: OnceLock<Regex> = OnceLock::new();
    let re = RE.get_or_init(|| Regex::new(r"(#\d+|\b[A-Z][A-Z0-9]+-\d+\b)").unwrap());
    let mut out: Vec<String> = Vec::new();
    for m in re.find_iter(text) {
        let s = m.as_str().to_string();
        if !out.contains(&s) {
            out.push(s);
        }
    }
    out
}

/// Vendored output and lockfiles: real changes, but noise for coupling/area.
fn is_noise(path: &str) -> bool {
    let p = path.replace('\\', "/");
    p.starts_with("target/")
        || p.contains("node_modules/")
        || p.starts_with(".venv/")
        || p.starts_with("webapp/dist")
        || p.ends_with("Cargo.lock")
        || p == "uv.lock"
}

/// Keep only *substantive* commits unless `include_all` is set: code changes
/// ([Evolution]/[Stability]) and major decisions ([Decision]). The whole
/// [Dependency] class — releases, version bumps, CI tweaks, and documentation
/// plumbing — is dropped by default so the graph reads as the project's design
/// history, not its mechanical churn. Design docs survive because they classify
/// as [Decision], not [Dependency].
#[must_use]
fn filter_commits(commits: Vec<RawCommit>, include_all: bool) -> Vec<RawCommit> {
    if include_all {
        return commits;
    }
    commits.into_iter().filter(is_substantive).collect()
}

/// True unless the commit classifies as [Dependency] (the non-code,
/// non-decision class: release/bump/CI/packaging/trivial-docs).
#[must_use]
fn is_substantive(c: &RawCommit) -> bool {
    let signal: Vec<String> = c.files.iter().filter(|f| !is_noise(f)).cloned().collect();
    classify(c, &signal) != NodeClass::Dependency
}

fn is_design_doc(path: &str) -> bool {
    let p = path.to_ascii_lowercase();
    if !p.starts_with("docs/") || !p.ends_with(".md") {
        return false;
    }
    ["spec", "architecture", "scope", "prd", "design"]
        .iter()
        .any(|kw| p.contains(kw))
}

/// Turn a commit subject into a filesystem-friendly slug, dropping the
/// conventional-commit prefix.
fn slugify(subject: &str) -> String {
    // Strip a leading `type(scope)!: ` prefix for a cleaner slug.
    let body = conventional_re().replace(subject, "").trim().to_string();
    let base = if body.is_empty() { subject } else { &body };

    let mut slug = String::new();
    let mut prev_dash = false;
    for ch in base.chars() {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch.to_ascii_lowercase());
            prev_dash = false;
        } else if !prev_dash {
            slug.push('-');
            prev_dash = true;
        }
    }
    let slug = slug.trim_matches('-');
    // Trim to ~48 chars on a dash boundary.
    let trimmed = if slug.len() > 48 {
        let cut = slug[..48].rfind('-').unwrap_or(48);
        &slug[..cut]
    } else {
        slug
    };
    if trimmed.is_empty() {
        "commit".to_string()
    } else {
        trimmed.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn commit(sha: &str, subject: &str, body: &str, files: &[&str]) -> RawCommit {
        RawCommit {
            sha: format!("{sha}00000000000000000000000000000000"),
            short_sha: sha.to_string(),
            author: "Test".into(),
            date: "2026-01-01".into(),
            subject: subject.into(),
            body: body.into(),
            files: files.iter().map(|s| s.to_string()).collect(),
        }
    }

    #[test]
    fn classifies_conventional_prefixes() {
        assert_eq!(
            classify(&commit("a", "feat: x", "", &[]), &[]),
            NodeClass::Evolution
        );
        assert_eq!(
            classify(&commit("a", "fix: x", "", &[]), &[]),
            NodeClass::Stability
        );
        assert_eq!(
            classify(&commit("a", "refactor: x", "", &[]), &[]),
            NodeClass::Decision
        );
        assert_eq!(
            classify(&commit("a", "chore: x", "", &[]), &[]),
            NodeClass::Dependency
        );
    }

    #[test]
    fn breaking_change_is_a_decision() {
        assert_eq!(
            classify(&commit("a", "feat!: x", "", &[]), &[]),
            NodeClass::Decision
        );
        assert_eq!(
            classify(&commit("a", "feat: x", "BREAKING CHANGE: y", &[]), &[]),
            NodeClass::Decision
        );
    }

    #[test]
    fn design_docs_are_decisions() {
        let f = vec!["docs/ARCHITECTURE.md".to_string()];
        assert_eq!(
            classify(
                &commit("a", "docs: update", "", &["docs/ARCHITECTURE.md"]),
                &f
            ),
            NodeClass::Decision
        );
    }

    #[test]
    fn area_picks_the_busiest() {
        let files = vec![
            "crates/dex-cli/src/main.rs".to_string(),
            "crates/dex-cli/src/commands/init.rs".to_string(),
            "docs/SPEC.md".to_string(),
        ];
        assert_eq!(area_of(&files), FunctionalArea::Cli);
    }

    #[test]
    fn scope_drives_area() {
        assert_eq!(scope_of("feat(mcp): add serve"), Some("mcp".to_string()));
        assert_eq!(area_from_scope("mcp"), Some(FunctionalArea::McpAi));
        assert_eq!(
            area_from_scope("scaffold"),
            Some(FunctionalArea::Scaffolding)
        );
        assert_eq!(area_from_scope("nonsense"), None);
    }

    #[test]
    fn loose_keyword_handles_slash_titles() {
        assert_eq!(
            loose_keyword("Chore/release v0.1.4"),
            Some(NodeClass::Dependency)
        );
        assert_eq!(
            loose_keyword("Feat/user config"),
            Some(NodeClass::Evolution)
        );
        assert_eq!(loose_keyword("Random subject"), None);
    }

    #[test]
    fn area_routes_template_engine() {
        assert_eq!(
            FunctionalArea::of_path("crates/dex-core/src/template/engine.rs"),
            FunctionalArea::TemplateEngine
        );
    }

    #[test]
    fn slugify_drops_prefix_and_cleans() {
        assert_eq!(
            slugify("feat(core): Add Template Caching!"),
            "add-template-caching"
        );
        assert_eq!(slugify("fix: bug #42 in core.rs"), "bug-42-in-core-rs");
    }

    #[test]
    fn parses_issue_refs() {
        let refs = parse_issue_refs("fix: resolve #42 and PROJ-7 (#43)");
        assert!(refs.contains(&"#42".to_string()));
        assert!(refs.contains(&"PROJ-7".to_string()));
        assert!(refs.contains(&"#43".to_string()));
    }

    #[test]
    fn noise_is_filtered() {
        assert!(is_noise("target/debug/foo"));
        assert!(is_noise("Cargo.lock"));
        assert!(is_noise("webapp/node_modules/x/y.js"));
        assert!(!is_noise("crates/dex-core/src/lib.rs"));
    }

    #[test]
    fn dependency_class_is_filtered_by_default() {
        // Dropped: the [Dependency] class — releases, bumps, CI, trivial docs.
        let release = commit("d000000", "chore: release v0.2.1", "", &["Cargo.toml"]);
        let bump = commit(
            "d000001",
            "chore: bump version to v0.3.0",
            "",
            &["Cargo.toml"],
        );
        let ci = commit(
            "d000002",
            "ci: add workflow_dispatch",
            "",
            &[".github/workflows/x.yml"],
        );
        let trivial_docs = commit("d000003", "docs: tweak readme", "", &["README.md"]);
        // Kept: code and major decisions.
        let feat = commit("c000000", "feat: x", "", &["crates/dex-core/src/lib.rs"]);
        let fix = commit(
            "c000001",
            "fix: y",
            "",
            &["crates/dex-core/src/scaffold.rs"],
        );
        let design = commit("c000002", "docs: add PRD", "", &["docs/prd-graph.md"]);

        assert!(!is_substantive(&release));
        assert!(!is_substantive(&bump));
        assert!(!is_substantive(&ci));
        assert!(!is_substantive(&trivial_docs));
        assert!(is_substantive(&feat));
        assert!(is_substantive(&fix));
        assert!(is_substantive(&design), "design docs are major decisions");

        let all = vec![
            release.clone(),
            trivial_docs.clone(),
            feat.clone(),
            fix.clone(),
            design.clone(),
        ];
        let kept = filter_commits(all.clone(), false);
        assert_eq!(kept.len(), 3, "code + decisions survive the default filter");
        // include_all keeps everything.
        assert_eq!(filter_commits(all, true).len(), 5);
    }

    #[test]
    fn co_occurrence_links_shared_files() {
        let commits = vec![
            commit(
                "aaaaaaa",
                "feat: a",
                "",
                &["crates/dex-core/src/scaffold.rs"],
            ),
            commit(
                "bbbbbbb",
                "fix: b",
                "",
                &["crates/dex-core/src/scaffold.rs"],
            ),
        ];
        let nodes = build_nodes(&commits);
        // The second (fix) node should reference the first via co-occurrence or modified-by.
        assert!(
            nodes[1]
                .edges
                .iter()
                .any(|e| e.target_stem == nodes[0].stem),
            "expected an edge from the fix node back to the feature node"
        );
    }
}
