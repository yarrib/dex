---
id: context-map
title: context-map
kind: module
summary: Emits a machine-readable .context-map.json after scaffolding, optimized for LLM consumption.
related:
  - dex-core: part-of
  - scaffold: reads
  - config: uses
  - error: uses
---

`crates/dex-core/src/context_map.rs` writes a `.context-map.json` into a freshly
scaffolded project. It is a machine-readable index optimized for AI agents: it
tells them what was created, the **role** of each file (e.g. `entry_point`,
`config`, `test`), where to start editing, and the project's tasks.

Roles come from template file-rule annotations (`context_role`,
`context_description`) when present, otherwise they are inferred from path
heuristics. Writing the map is best-effort and non-fatal.

> Note: this is the *per-generated-project* context map. It is the conceptual
> cousin of this repo's own `.context/` knowledge graph, but operates on
> scaffolded output rather than the dex codebase itself.
