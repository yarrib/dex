---
id: skills
title: skills
kind: concept
summary: Installable bundles of agent instructions/commands, discovered and installed by dex-core and shipped in skills/.
related:
  - dex-core: part-of
  - cli-commands: exposed-by
  - error: uses
---

Skills are reusable bundles of agent guidance/commands that dex can install into
a project. The core logic lives in `crates/dex-core/src/skills/`
(`registry.rs` for discovery, `installer.rs` for installation, `manifest.rs` for
parsing), and the built-in skill bundles ship in the top-level `skills/`
directory (`agent-dev`, `databricks`, `default`).

The `dex skills` command (`crates/dex-cli/src/commands/skills.rs`) exposes
listing and installation to users. See `docs/skills-authoring.md`.
