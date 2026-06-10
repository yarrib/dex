---
sha: 28f4530bffbc46e2b4616a935fd397075f39dfb7
short_sha: 28f4530
author: Claude
date: 2026-06-10
class: [Evolution]
area: Docs, CI & Release
tags: [#evolution]
---

# [Evolution] feat(context): export project-memory graph to the docs site

**Commit:** `28f4530` · **Author:** Claude · **Date:** 2026-06-10 · **Area:** Docs, CI & Release

Add `dex context export`: renders the committed `.context/wiki/` into
mdBook-ready pages (default docs/wiki/), rewriting Obsidian `[[wikilinks]]`
to relative links and injecting a SUMMARY.md nav section (grouped by
functional area) between markers.

The docs-deploy workflow now builds dex, runs `context sync` + `export`
before mdbook build, so the graph is browsable on GitHub Pages alongside
Obsidian/Logseq. Generated docs/wiki/ is gitignored like the changelog.

## Changed files

- `.github/workflows/docs.yml`
- `.gitignore`
- `crates/dex-cli/src/commands/context.rs`
- `crates/dex-core/src/context_graph/mod.rs`
- `crates/dex-core/src/context_graph/render.rs`
- `crates/dex-core/src/lib.rs`
- `docs/SPEC.md`
- `docs/SUMMARY.md`

## Relationships

- **influenced-by** → [[c04db4f-add-prd-for-org-validated-skills-mcp-server]] (Docs, CI & Release)
- **co-occurrence** → [[bf31033-add-rust-project-memory-knowledge-graph-engine]] (8 shared files)
- **co-occurrence** → [[0e067c2-add-workflow-dispatch-to-docs-deploy-workflow]] (4 shared files)
- **co-occurrence** → [[9f25600-add-context-knowledge-graph-and-render-it-on]] (3 shared files)
