---
id: dex-toml
title: dex.toml
kind: config
summary: The per-project config file. Defines tasks and pass-through commands.
related:
  - config: parsed-by
  - passthrough: defines
  - cli-commands: read-by
---

`dex.toml` is the project-level configuration file dex writes into scaffolded
projects and reads on subsequent commands. It is TOML by rule.

Key tables:

- `[project]` — name and source template.
- `[tasks.*]` — named commands runnable via `dex run`.
- `[passthrough]` — subcommands delegated to external CLIs.

It is parsed by the `config` module and consumed by the CLI command layer and
`context-map` generation.
