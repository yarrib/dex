---
id: template-engine
title: template engine
kind: module
summary: Renders Jinja2 templates with minijinja — engine wrapper, registry, and variable handling.
related:
  - dex-core: part-of
  - minijinja: uses
  - templates: renders
  - template-manifest: reads
  - scaffold: used-by
  - error: uses
---

`crates/dex-core/src/template/` is the rendering subsystem. dex extensibility is
template-driven, and templates use Jinja2 syntax (`.j2` file extension), familiar
to Python users.

Submodules:

- `engine.rs` — a `minijinja::Environment` wrapper that renders strings/files.
- `registry.rs` — discovers and loads templates (built-in via `include_dir`, plus
  directory and remote git registries).
- `variables.rs` — variable specs, defaults, and validation.
- `manifest.rs` — deserializes `template.toml`.

Built-in templates are embedded at compile time. The engine produces rendered
content; `scaffold` writes it to disk.
