---
name: project-memory-engine
description: Build or refresh a linked knowledge graph in `.context/wiki/` for this repo — generates per-commit Markdown nodes from git history, code, and docs, stitches them with wiki-style edges, and writes a functional-area INDEX.md. Use when the user says "sync project memory", "rebuild wiki", "refresh .context/wiki", or asks to regenerate the project evolution knowledge graph.
---

# Project Memory Engine

Build a **Project Evolution Knowledge Graph** for dex from git, code, and docs
and write it to `.context/wiki/`. Produces one Markdown node per significant
commit plus an `INDEX.md` "city map" grouped by functional area.

dex ships a **deterministic Rust engine** for this — `dex context sync`. Prefer
it; your job is to run it and then add the *semantic* judgment heuristics can't
make. Never hand-build the whole graph when the binary is available.

## Output contract

- Directory: `.context/wiki/` (create if missing).
- One node per significant commit: `<short_sha>-<slug>.md`.
- One index file: `.context/wiki/INDEX.md`.
- A user manual at `.context/USER_MANUAL.md`.
- Every node uses `[[wikilink]]` edges to other nodes.

## Preferred path — run the engine

```bash
dex context sync            # incremental: only new commits get nodes
dex context sync --rebuild  # full regeneration
dex context sync --limit 100  # cap history on a first run
```

The engine does all of Phase 1–4 below deterministically: git parsing,
conventional-commit classification, co-change analysis, and indexing. If `dex`
is not on PATH, build it (`cargo build`) or fall back to the manual phases.

## Then — semantic enrichment (your value-add)

After the engine runs, review the graph and improve what heuristics miss:

- Upgrade misclassified nodes. The engine classifies from commit prefixes; you
  can read the diff/body and correct `[Decision]` vs `[Evolution]` etc. (edit
  the node's frontmatter `class:` and title).
- Add `[[influenced-by]]` / `[[implemented-in]]` edges that require reading
  *why* a change happened, not just which files moved together.
- Add a one-line rationale to important `[Decision]` nodes ("chose minijinja
  because Python users know Jinja2").

The engine is incremental and will **not** overwrite your hand edits — only
`--rebuild` does. So enrich freely.

## Phase 1 — Structural analysis (code + docs)

Scan dex's actual source tree (a Cargo workspace, no top-level `/src`):

- `crates/dex-core/` — the library; all business logic, no UI. Key modules:
  `config.rs`, `error.rs`, `scaffold.rs`, `apply_trait.rs`, `context_map.rs`,
  `context_graph/` (this engine), `mcp.rs`, `template/`, `skills/`, `traits/`.
- `crates/dex-cli/` — the binary: `main.rs`, `output.rs`, `commands/*` (init,
  add, agent, run, mcp, skills, templates, context, passthrough).
- `crates/dex-py/` — optional PyO3 bindings.
- `templates/` — built-in project templates (embedded at compile time).
- `traits/`, `skills/` — composable capability bundles and skill packs.
- `webapp/` — browser scaffolding app.

Read `docs/` for Architecturally Significant Requirements. Known anchors:
`docs/SPEC.md`, `docs/ARCHITECTURE.md`, `docs/SCOPE.md`, `docs/prd-*.md`,
`docs/internal/*`. Read the top-level `CLAUDE.md` for the architectural rules
and build commands before classifying.

## Phase 2 — Temporal mapping (git logs)

The engine runs the equivalent of:

```bash
git log --no-merges --date=short --pretty=format:'%H|%an|%ad|%s'
git log --no-merges --name-only --pretty=format:'==%H=='
```

For each commit, classify into one of:

- `[Decision]` — architectural pivot (`refactor:`, breaking `!`, design-doc commits).
- `[Evolution]` — major feature / capability addition (`feat:`).
- `[Stability]` — bug fix, hardening, resilience (`fix:`, `perf:`, `test:`).
- `[Dependency]` — environment / config / packaging (`chore:`, `ci:`, `build:`, `deps:`).

Skip vendored/lock noise when finding co-change clusters: `target/`,
`webapp/node_modules/`, `webapp/dist/`, `.venv/`, `Cargo.lock`, `uv.lock`. Skip
merge commits.

## Phase 3 — Semantic stitching

Edges between nodes:

- `[[influenced-by]]` — a feature builds on a prior `[Decision]`.
- `[[modified-by]]` — a fix alters a design pattern; for every `[Stability]`
  node, link to the `[Decision]`/`[Evolution]` node whose module it touches.
- `[[implemented-in]]` — between a design-doc node and the code that realises it.
- `[[resolved-by]]` — between Issue/PR IDs (`#NN`) in messages and the change.
- `[[co-occurrence]]` — files that consistently change together.

Known co-change clusters in this repo (the engine surfaces these in INDEX.md):

- **CLI dispatch:** `crates/dex-cli/src/commands/mod.rs` + `main.rs` (every new subcommand).
- **Core API surface:** `crates/dex-core/src/lib.rs` + `config.rs`.
- **Scaffold + manifest:** `crates/dex-core/src/scaffold.rs` + `template/manifest.rs`.
- **Release plumbing:** `.github/workflows/release.yml` + `Makefile`.

## Phase 4 — Indexing

`INDEX.md` is a Knowledge Map grouped by functional area, not chronology:

1. Foundation & Architecture
2. Template Engine & Rendering
3. Scaffolding & Project Generation
4. CLI & Interfaces
5. Skills, Traits & Extensibility
6. MCP & AI Integration
7. Templates & Built-in Content
8. Docs, CI & Release

Include: a "Reading order for new agents" section, the co-change clusters, and a
node-type legend (`[Decision]` / `[Evolution]` / `[Stability]` / `[Dependency]`).

## Incremental refresh

If `.context/wiki/` already exists (`dex context sync` without `--rebuild`):

- Keep existing nodes; only create nodes for commits not already on disk.
- Rewrite `INDEX.md` in place so it reflects the full graph.
- Never rewrite a node that already exists (preserves hand edits).

## Door for local-LM assist (future)

The semantic enrichment step is where a local model can help offline: feed it a
node's diff + body and have it propose a class and `[[influenced-by]]` targets,
then apply the suggestions to the node frontmatter. The deterministic engine
output is the stable substrate; LM assist is an optional layer on top.
