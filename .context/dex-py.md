---
id: dex-py
title: dex-py
kind: crate
summary: Optional PyO3 bindings exposing dex-core to Python. Not required for the native binary.
related:
  - dex-core: wraps
  - PyO3: uses
---

`crates/dex-py/` is a thin, **optional** FFI layer that exposes `dex-core` to
Python via PyO3. It exists for backwards compatibility and Python interop, but
is not part of the primary distribution — the native `dex` binary from `dex-cli`
needs no Python runtime.

Because it wraps `dex-core`, it inherits the same business logic. Its job is
type conversion across the FFI boundary (Rust results/errors ↔ Python
objects/exceptions), not new behavior.
