---
id: template-manifest
title: template.toml
kind: config
summary: Per-template manifest declaring variables, file rules, and context annotations.
related:
  - template-engine: read-by
  - config: parsed-by
  - templates: describes
---

`template.toml` is the manifest that accompanies each template. It declares:

- **variables** — names, defaults, prompts, and validation.
- **file rules** — how source files map to output paths, including optional
  `context_role` / `context_description` annotations that flow into the
  generated `.context-map.json`.
- **metadata** — template name, description, version, and minimum dex version.

It is deserialized by `template/manifest.rs` and drives both rendering and the
per-project context map.
