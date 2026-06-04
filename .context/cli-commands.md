---
id: cli-commands
title: cli commands
kind: concept
summary: The user-facing subcommands (init, add, agent, run, mcp, skills, templates, passthrough) under dex-cli.
related:
  - dex-cli: part-of
  - dex-core: calls
  - dex-toml: reads
---

The subcommands a user invokes, implemented in `crates/dex-cli/src/commands/`
and dispatched from `main.rs`:

- `init` — scaffold a new project from a template.
- `add` — add traits/capabilities to an existing project.
- `agent` — scaffold AI agent projects (`dex agent new`).
- `run` — run a task defined in `dex.toml`.
- `mcp` — serve dex over the Model Context Protocol.
- `skills` — list/install skill bundles.
- `templates` — list/inspect available templates.
- `passthrough` — delegate to configured external CLIs.

Each command is a thin shell: parse args, call `dex-core`, render results.
