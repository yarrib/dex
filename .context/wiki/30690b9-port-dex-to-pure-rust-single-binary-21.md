---
sha: 30690b9633c05da429f7e01eb1ec5dcb0a2b28ef
short_sha: 30690b9
author: yarrib
date: 2026-03-27
class: [Evolution]
area: CLI & Interfaces
tags: [#evolution]
---

# [Evolution] feat: port dex to pure Rust single binary (#21)

**Commit:** `30690b9` · **Author:** yarrib · **Date:** 2026-03-27 · **Area:** CLI & Interfaces

Add dex-cli binary crate with clap CLI, replacing the Python CLI layer.
All user interaction (prompts, output formatting, pass-throughs) now
handled in Rust via dialoguer, console, and std::process::Command.

- New crate: dex-cli with `dex init`, `dex agent new`, and passthrough
support
- Move config loading/merging (user + project) from Python to dex-core
- Add standards file loading, remote template resolution to dex-core
- Update CLAUDE.md for pure-Rust architecture
- dex-py kept as optional for backwards compat but not required

Dependencies added: clap, dialoguer, console, indicatif, dirs,
shellexpand

https://claude.ai/code/session_018DhTR7Sg2N5iyPDAi4vggq

Co-authored-by: Claude <noreply@anthropic.com>

## Changed files

- `CLAUDE.md`
- `Cargo.toml`
- `crates/dex-cli/Cargo.toml`
- `crates/dex-cli/src/commands/agent.rs`
- `crates/dex-cli/src/commands/init.rs`
- `crates/dex-cli/src/commands/mod.rs`
- `crates/dex-cli/src/commands/passthrough.rs`
- `crates/dex-cli/src/main.rs`
- `crates/dex-cli/src/output.rs`
- `crates/dex-core/Cargo.toml`
- `crates/dex-core/src/config.rs`
- `crates/dex-core/src/lib.rs`

## Relationships

- **implemented-in** → [[31c18f8-feat-documentation-12]]
- **co-occurrence** → [[0e067c2-add-workflow-dispatch-to-docs-deploy-workflow]] (5 shared files)
- **co-occurrence** → [[7719579-inline-variables-format-order-field-and]] (2 shared files)
- **co-occurrence** → [[7b9ade0-release-v0-1-0-17]] (1 shared file)
- **resolved-by** → `#21` _(this commit)_
