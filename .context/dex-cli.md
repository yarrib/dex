---
id: dex-cli
title: dex-cli
kind: crate
summary: The binary crate. Owns all user interaction — argument parsing, prompts, and terminal output.
related:
  - dex-core: depends-on
  - cli-commands: contains
  - dex-toml: reads
  - clap: uses
  - dialoguer: uses
  - console: uses
---

`crates/dex-cli/` produces the `dex` binary — the primary distribution. It owns
**all** user interaction: argument parsing (`clap` derive), interactive prompts
(`dialoguer`), and output formatting/styling (`console`, `indicatif`).

It is a thin layer: it parses input, calls into `dex-core` for the actual work,
then renders the returned data or errors. `src/main.rs` dispatches to command
modules under `src/commands/` (init, add, agent, run, mcp, skills, templates,
passthrough). Output helpers live in `src/output.rs`.
