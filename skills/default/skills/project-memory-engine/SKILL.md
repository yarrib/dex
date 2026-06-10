---
name: project-memory-engine
description: Build or refresh a linked knowledge graph of a project's evolution in `.context/wiki/` — per-commit Markdown nodes from git history, stitched with wiki-style edges and a functional-area INDEX.md. Use when the user says "sync project memory", "rebuild wiki", "refresh .context/wiki", or asks to regenerate the project evolution knowledge graph.
---

# Project Memory Engine

Build a **Project Evolution Knowledge Graph** from this project's git history and
write it to `.context/wiki/`: one Markdown node per significant commit, plus an
`INDEX.md` "city map" grouped by functional area. It renders as a live graph in
Obsidian / Logseq / VS Code (Foam) and gives a fresh agent the project's
accumulated design judgment the moment it opens the repo.

dex provides a **deterministic Rust engine** for this — `dex context sync`.
Prefer it; your job is to run it, then add the semantic judgment heuristics
can't make.

## Preferred path — run the engine

```bash
dex context sync             # incremental: only new commits get nodes
dex context sync --rebuild   # full regeneration
dex context sync --limit 200 # cap history on a first run of a large repo
```

This produces, under `.context/`:

- `wiki/<short_sha>-<slug>.md` — one node per commit, classified
  `[Decision]` (architectural pivot) / `[Evolution]` (feature) /
  `[Stability]` (fix/hardening) / `[Dependency]` (config/packaging).
- `wiki/INDEX.md` — nodes grouped by functional area, with a node-type legend,
  a "reading order for new agents", and co-change coupling clusters.
- `USER_MANUAL.md` — how to read and use the graph.

Edges use Obsidian `[[wikilinks]]`: `influenced-by`, `modified-by`,
`implemented-in`, `co-occurrence`, plus `resolved-by` for issue/PR references.

If `dex` is not on PATH, install it (`cargo install --path crates/dex-cli` in
the dex repo, or download a release binary) before running.

## Then — semantic enrichment (your value-add)

After the engine runs, review and improve what heuristics miss:

- **Correct classifications.** The engine classifies from commit prefixes; read
  the diff/body and fix `class:` in a node's frontmatter when it's wrong.
- **Add reasoning edges.** `[[influenced-by]]` and `[[implemented-in]]` often
  require understanding *why* a change happened — add those links by hand.
- **Capture rationale.** Add a sentence to important `[Decision]` nodes ("chose
  X over Y because …"). That insight then flows to every future AI session.

The engine is incremental and never overwrites existing node files (only
`--rebuild` does), so hand edits are safe.

## Cadence

- Run at the end of each milestone so status reports pull from a fresh graph.
- Run whenever a `[Decision]`-class change lands so future sessions inherit it.
- Point `CLAUDE.md` / `AGENTS.md` at `.context/wiki/INDEX.md` so agents consult
  the graph before non-trivial changes.
