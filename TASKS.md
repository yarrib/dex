# dex — Task Backlog

## In Progress

_(nothing active)_

## Backlog

### Features

- [ ] Add `dabs-dashboard` template — Databricks Lakeview dashboard (`resources/<name>_dashboard.yml`, YAML schema, deploy instructions)
- [ ] Add `dabs-genie-space` template — Genie space definition (`resources/<name>_genie_space.yml`, curated SQL, space config)
- [ ] Add `databricks-app-streamlit` template — Databricks Apps + Streamlit (`app.py`, `app.yaml`, deploy via `databricks apps deploy`)
- [ ] Add `databricks-app-react` template — Databricks Apps + React (`package.json`, Vite config, `src/App.tsx`, `app.yaml`, build + deploy)
- [ ] MCP tool stubs — `scaffold_project`, `scaffold_agent`, `get_template_variables` not yet implemented in `crates/dex-cli/src/commands/mcp.rs`
- [ ] Remote trait sources — `[traits.remotes]` in `dex.toml`/user config (prerequisite for `dex.lock`)

### Bugs

_(none known)_

### Infrastructure

- [ ] CI: update to Rust-only (remove Python CI steps now that Python layer is deprecated)
- [ ] Docs: update to reflect native binary distribution (remove PyO3/maturin references)

## Done

- [x] Initial Rust core — template engine, config, file I/O (`dex-core`)
- [x] Native binary CLI (`dex-cli`) — `dex init`, `dex add`, `dex skills`, `dex templates`, `dex mcp`, `dex run`
- [x] `default`, `python-package`, `dabs-package`, `dabs-etl`, `dabs-ml`, `dabs-aiagent` templates
- [x] Multi-variable scaffolding — all manifest variables prompted interactively
- [x] `dex.toml` written by `dex init` (unblocks `dex add`)
- [x] `dex templates list/show` — discover built-in and remote templates
- [x] Skills system — `dex skills init/list/add/sync`, pack manifests, multi-target install
- [x] Traits system — `dex add <trait>`, embedded `ci-github` and `docker` traits
- [x] MCP server — `dex mcp serve`, `.mcp.json` wiring, `scaffold_agent` tool
- [x] User config — `~/.config/dex/config.toml` + `dex.toml`, remote template sources, presets, standards
- [x] Devcontainer + ai-dev-kit integration — profile-based skill setup, `scripts/setup_dev_kit.sh`
- [x] GitHub Releases — `install.sh`, platform binaries (linux x86_64/aarch64, macos x86_64/aarch64)
- [x] Docs site — mkdocs-material, versioned with mike, GitHub Pages
- [x] CI/CD — `ci.yml`, `release.yml` (cross-compiled musl), `docs.yml`
