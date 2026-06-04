---
id: dex-core
title: dex-core
kind: crate
summary: The Rust library holding all business logic. No UI, no terminal output — it returns data for the CLI to render.
related:
  - config: contains
  - template-engine: contains
  - scaffold: contains
  - context-map: contains
  - mcp: contains
  - skills: contains
  - traits: contains
  - error: contains
  - dex-cli: consumed-by
  - dex-py: consumed-by
---

`crates/dex-core/` is the heart of dex. Per the architectural rules it contains
**all** business logic and has **no UI**: no colors, prompts, or spinners. It
returns plain data and propagates `thiserror`-based errors with `?`; callers
(the CLI) decide how to render them.

The public API surface lives in `src/lib.rs`; implementation is split into
submodules (`config`, `template`, `scaffold`, `context_map`, `mcp`, `skills`,
`traits`, `error`). Keeping the core pure and UI-free is what makes it testable
and reusable across the CLI binary and the optional Python bindings.
