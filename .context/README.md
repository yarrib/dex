# `.context/` — repo knowledge graph

This folder is a hand-maintained **knowledge graph of the dex codebase**, kept
for posterity. Each file describes one *entity* (a crate, module, concept, or
artifact) and the relationships that connect it to other entities.

It is the durable, human- and LLM-readable map of "what exists and how it fits
together". The rendered, browsable version lives on the docs site:
[Knowledge Graph](../docs/knowledge-graph.md) (published to GitHub Pages).

## File format

Each entity is a markdown file with a small frontmatter block followed by a
prose summary:

```markdown
---
id: scaffold
title: scaffold
kind: module
summary: One-sentence description shown in the graph.
related:
  - template-engine: uses
  - context-map: produces
  - dex-core: part-of
---

Longer prose explaining the entity, its responsibilities, and key files.
```

Frontmatter fields:

| Field     | Meaning                                                               |
|-----------|-----------------------------------------------------------------------|
| `id`      | Stable identifier (kebab-case). Used for graph nodes and links.       |
| `title`   | Human label shown on the node.                                        |
| `kind`    | One of: `crate`, `module`, `concept`, `artifact`, `config`, `meta`.   |
| `summary` | One sentence. Shown beneath the node's section.                       |
| `related` | List of `target: relationship` edges. `target` may be another entity's `id` or an external name (e.g. `minijinja`). External targets render as grey nodes. |

## Adding or updating an entity

1. Add or edit a file in `.context/` following the format above.
2. Regenerate the graph page:

   ```bash
   python3 scripts/gen_context_graph.py
   ```

   This rewrites `docs/knowledge-graph.md`. The generator is stdlib-only.
3. Commit both the `.context/` change and the regenerated page.

CI also regenerates the page on every docs deploy, so the published graph never
drifts from these files.

## Rendering

The graph is drawn with [Mermaid](https://mermaid.js.org/) inside the mdBook
docs site (`mdbook-mermaid` preprocessor). For a **local** docs build you need
the mermaid assets once:

```bash
cargo install mdbook mdbook-mermaid
mdbook-mermaid install .     # drops mermaid.min.js + mermaid-init.js (gitignored)
python3 scripts/gen_context_graph.py
mdbook build                 # output in ./book
```
