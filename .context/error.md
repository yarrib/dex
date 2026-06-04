---
id: error
title: error
kind: module
summary: The thiserror-based error type for dex-core. Errors are propagated, never panicked.
related:
  - dex-core: part-of
  - thiserror: uses
---

`crates/dex-core/src/error.rs` defines `DexError`, the library's error type,
built with `thiserror` (never `anyhow` in the core). Every fallible operation in
`dex-core` returns `Result<_, DexError>` and uses `?` for propagation — no
`unwrap()` or `expect()` in library code.

Variants carry context (e.g. `Io { path, source }`). The CLI layer maps these to
formatted, user-facing messages, and `dex-py` maps them to Python exceptions.
