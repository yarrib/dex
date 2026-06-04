---
id: docs-site
title: docs site
kind: meta
summary: The mdBook documentation site, auto-deployed to GitHub Pages — and home of this rendered knowledge graph.
related:
  - dex-cli: documents
  - dex-core: documents
  - templates: documents
  - mdBook: built-by
---

The `docs/` directory is an [mdBook](https://rust-lang.github.io/mdBook/) site.
`book.toml` configures it (`src = "docs"`) and `docs/SUMMARY.md` defines the
navigation. The `.github/workflows/docs.yml` workflow builds it and deploys to
the `gh-pages` branch via `peaceiris/actions-gh-pages` on every push to `main`.

The same pipeline regenerates `docs/changelog.md` (git-cliff) and
`docs/knowledge-graph.md` (`scripts/gen_context_graph.py`), then renders Mermaid
diagrams via the `mdbook-mermaid` preprocessor. This entity's own graph is
published here for posterity.
