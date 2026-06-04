---
id: scaffold
title: scaffold
kind: module
summary: Orchestrates turning a template into a directory tree — creates dirs and renders files.
related:
  - dex-core: part-of
  - template-engine: uses
  - context-map: produces
  - error: uses
---

`crates/dex-core/src/scaffold.rs` is the orchestrator behind `dex init`. Given a
loaded template and resolved variables, it creates the directory structure and
renders each file through the template engine to disk.

It returns a `ScaffoldResult` (`files_created`, `directories_created`,
`on_success`) which the CLI renders, and which `context-map` reads to emit the
project's `.context-map.json`. Related logic for applying optional capability
bundles lives in `apply_trait.rs`.
