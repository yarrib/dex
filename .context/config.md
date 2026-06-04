---
id: config
title: config
kind: module
summary: Parses and validates TOML configuration — project, template, and user config.
related:
  - dex-core: part-of
  - dex-toml: parses
  - template-manifest: parses
  - error: uses
---

`crates/dex-core/src/config.rs` handles all TOML configuration parsing and
merging. dex is TOML-only by rule — no YAML or JSON for config.

It reads three kinds of config:

- **Project config** — `dex.toml` (tasks and pass-through commands).
- **Template manifests** — `template.toml` (variables, file rules).
- **User config** — `~/.config/dex/config.toml`.

It exposes typed structs (e.g. project config with a `tasks` map) consumed by
`scaffold`, `context-map`, and the CLI command layer.
