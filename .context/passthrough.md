---
id: passthrough
title: passthrough
kind: concept
summary: Config-driven delegation to external CLIs (databricks, az, aws, git) defined in dex.toml.
related:
  - dex-toml: defined-in
  - cli-commands: run-by
  - dex-core: part-of
---

Pass-throughs let orgs extend dex without writing code: a `[passthrough]` table
in `dex.toml` maps a dex subcommand to an external CLI invocation, executed via
`std::process::Command`. This is one of the two primary extensibility mechanisms
(the other being templates).

The CLI's `commands/passthrough.rs` resolves and runs these delegations, wiring
through arguments to tools like `databricks`, `az`, `aws`, or `git`.
