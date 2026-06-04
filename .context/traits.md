---
id: traits
title: traits
kind: concept
summary: Optional capability add-ons (e.g. docker, CI, notebooks) layered onto a project, defined in traits/.
related:
  - dex-core: part-of
  - scaffold: applied-by
  - template-engine: uses
  - error: uses
---

Traits are composable capability bundles applied on top of a scaffolded project
— for example adding a `Dockerfile`, GitHub CI, or notebook support. Core logic
lives in `crates/dex-core/src/traits/` (`registry.rs`, `manifest.rs`) plus
`apply_trait.rs`, which renders trait files through the template engine.

The built-in traits ship in the top-level `traits/` directory: `ci-github`,
`docker`, and `notebook`, each with a `trait.toml` manifest and `files/`.
