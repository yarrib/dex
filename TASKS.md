# dex — Task Backlog

## In Progress

_(nothing active)_

## Backlog

### Features

- [ ] Remote trait sources — `[traits.remotes]` in `dex.toml`/user config (prerequisite for `dex.lock`)
- [ ] `dex.lock` — reproducible trait & skill resolution (see `docs/internal/prd-dex-lock.md`)
- [ ] WASM Compatibility — compile `dex-core` to `wasm32-unknown-unknown` (see `docs/internal/prd-ai-ready-scaffolding.md`)
- [ ] Context-Map: populate `tasks` from scaffolded `dex.toml` at write time
- [ ] MCP: `apply_trait` tool (expose `dex add` via MCP)

### Infrastructure

- [ ] Docs: update to reflect native binary distribution (remove any remaining PyO3/maturin references)

## Done

- [x] Initial Rust core — template engine, config, file I/O (`dex-core`)
- [x] Native binary CLI (`dex-cli`) — `dex init`, `dex add`, `dex skills`, `dex templates`, `dex mcp`, `dex run`
- [x] `dabs-platform` — profile-driven Databricks Asset Bundle template (presets: `data_eng`, `mlops`, `agents`, `extraction_agents`); supersedes `dabs-package`/`dabs-etl`/`dabs-ml`/`dabs-aiagent`
- [x] `default`, `python-package`, `dabs-package`, `dabs-etl`, `dabs-ml`, `dabs-aiagent` templates
- [x] Multi-variable scaffolding — all manifest variables prompted interactively
- [x] `dex.toml` written by `dex init` (unblocks `dex add`)
- [x] `dex templates list/show` — discover built-in and remote templates
- [x] Skills system — `dex skills init/list/add/sync`, pack manifests, multi-target install
- [x] Traits system — `dex add <trait>`, embedded `ci-github` and `docker` traits
- [x] `notebook` built-in trait — `dex add notebook` adds a Databricks notebook (percent format)
- [x] MCP server — `dex mcp serve`, `.mcp.json` wiring, full `scaffold_project`, `get_template_variables`, `list_templates` tools
- [x] MCP client wiring — `dex mcp install` writes/merges config for claude-code, claude-desktop, cursor, vscode/copilot, codex, zed, antigravity; install.sh hook
- [x] User config — `~/.config/dex/config.toml` + `dex.toml`, remote template sources, presets, standards
- [x] Devcontainer + ai-dev-kit integration — profile-based skill setup, `scripts/setup_dev_kit.sh`
- [x] GitHub Releases — `install.sh`, platform binaries (linux x86_64/aarch64, macos x86_64/aarch64)
- [x] Docs site — mkdocs-material, versioned with mike, GitHub Pages
- [x] CI/CD — `ci.yml`, `release.yml` (cross-compiled musl), `docs.yml`
- [x] `dabs-dashboard`, `dabs-genie-space`, `databricks-app-streamlit` templates
- [x] `databricks-app-react` template — Databricks Apps + Next.js (TypeScript, App Router)
- [x] Context-Map Generation — `.context-map.json` written after `dex init` for AI agent consumption
