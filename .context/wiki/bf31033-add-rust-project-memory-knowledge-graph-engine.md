---
sha: bf31033e955197abd65831f596e7fd4ef05e751e
short_sha: bf31033
author: Claude
date: 2026-06-04
class: [Evolution]
area: Docs, CI & Release
tags: [#evolution]
---

# [Evolution] feat(context): add Rust project-memory knowledge-graph engine

**Commit:** `bf31033` · **Author:** Claude · **Date:** 2026-06-04 · **Area:** Docs, CI & Release

Replace the entity-based Mermaid/mdBook prototype with a commit-based
*Project Evolution Knowledge Graph*, generated 100% in Rust.

New `dex context sync` command (core logic in dex-core::context_graph,
CLI in dex-cli) reads git history and writes `.context/wiki/`:
  - one Markdown node per significant commit (`<sha>-<slug>.md`),
    classified [Decision]/[Evolution]/[Stability]/[Dependency]
  - Obsidian-style [[wikilink]] edges: influenced-by, modified-by,
    implemented-in, co-occurrence, plus resolved-by issue refs
  - INDEX.md "city map" grouped by dex's 8 functional areas, with a
    node legend, reading order, and co-change coupling clusters
  - a USER_MANUAL.md explaining the graph for humans, devs, and agents
Incremental by default (only new commits get nodes; hand edits are
preserved); `--rebuild` regenerates everything.

Classification is deterministic (conventional-commit prefixes + scope
for area routing + breaking-change detection). The new
`project-memory-engine` skill drives the engine and adds a semantic
enrichment layer, with a documented door for local-LM assist.

CLAUDE.md and AGENTS.md gain an "Architectural History" pointer so
agents consult the graph before non-trivial changes. SPEC.md documents
the command. Renders live in Obsidian/Logseq/VS Code-Foam via wikilinks.

Removes the prior `.context/*.md` entity files, scripts/gen_context_graph.py,
docs/knowledge-graph.md, and the mdbook-mermaid wiring.

## Changed files

- `.claude/skills/project-memory-engine/SKILL.md`
- `.context/README.md`
- `.context/USER_MANUAL.md`
- `.context/cli-commands.md`
- `.context/config.md`
- `.context/context-map.md`
- `.context/dex-cli.md`
- `.context/dex-core.md`
- `.context/dex-py.md`
- `.context/dex-toml.md`
- `.context/docs-site.md`
- `.context/error.md`
- `.context/mcp.md`
- `.context/passthrough.md`
- `.context/scaffold.md`
- `.context/skills.md`
- `.context/template-engine.md`
- `.context/template-manifest.md`
- `.context/templates.md`
- `.context/traits.md`
- _…and 71 more_

## Relationships

- **influenced-by** → [[c04db4f-add-prd-for-org-validated-skills-mcp-server]] (Docs, CI & Release)
- **co-occurrence** → [[9f25600-add-context-knowledge-graph-and-render-it-on]] (24 shared files)
- **co-occurrence** → [[0e067c2-add-workflow-dispatch-to-docs-deploy-workflow]] (7 shared files)
- **co-occurrence** → [[7f7b8e0-batteries-included-assistant-agnostic-agent]] (5 shared files)
