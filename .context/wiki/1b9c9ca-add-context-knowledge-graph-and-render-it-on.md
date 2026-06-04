---
sha: 1b9c9ca1471b97a70e4fd00e2f8ad80504bec515
short_sha: 1b9c9ca
author: Claude
date: 2026-06-04
class: [Dependency]
area: Docs, CI & Release
tags: [#dependency]
---

# [Dependency] docs: add .context knowledge graph and render it on Pages

**Commit:** `1b9c9ca` · **Author:** Claude · **Date:** 2026-06-04 · **Area:** Docs, CI & Release

Introduce a hand-maintained `.context/` knowledge graph of the codebase:
one markdown file per entity (crate, module, concept, artifact) with
frontmatter declaring `related:` edges, plus a prose summary. This is the
durable, human- and LLM-readable map of the repo, kept for posterity.

Add `scripts/gen_context_graph.py` (stdlib-only) which reads the entity
files and generates `docs/knowledge-graph.md` — a Mermaid diagram with
clickable nodes plus per-entity sections. Reciprocal edges are collapsed
in the diagram for readability while text sections keep full relations.

Wire it into the existing mdBook -> GitHub Pages pipeline via the
mdbook-mermaid preprocessor (book.toml), a new SUMMARY.md entry, and
docs.yml steps that regenerate the graph and install mermaid assets on
every deploy so the published graph never drifts from `.context/`.

## Changed files

- `.context/README.md`
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
- `.github/workflows/docs.yml`
- `.gitignore`
- _…and 4 more_

## Relationships

- **implemented-in** → [[9013fda-dex-mcp-install-client-wiring-docs-55]]
- **co-occurrence** → [[8f177a0-migrate-from-mkdocs-to-mdbook-23]] (3 shared files)
- **co-occurrence** → [[0e067c2-add-workflow-dispatch-to-docs-deploy-workflow]] (2 shared files)
- **co-occurrence** → [[bacd088-add-dex-run-task-command-28]] (1 shared file)
