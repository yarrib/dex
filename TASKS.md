# dex — Task Backlog

## In Progress

_(nothing active)_

## Backlog

### Features
- [ ] Fix `init_command` variable prompting — currently only `project_name` is prompted; other manifest variables are skipped
- [ ] Fix slugifier for hyphenated package names — `table-anomaly-monitor` should become `table_anomaly_monitor` (valid Python identifier)
- [ ] Expose `system_prompt` and `claude_md` in `AgentScaffoldResultPy` PyO3 binding — fields are generated in Rust but dropped at the FFI boundary
- [ ] Implement MCP server tool stubs — `scaffold_project`, `scaffold_agent`, `get_template` are currently unimplemented in `mcp_server.py`
- [ ] Add `dabs-etl`, `dabs-ml`, `dabs-aiagent` templates (PRDs in `docs/prd-templates.md`)
- [ ] `_core.pyi` stub file — no type stubs exist for the PyO3 extension, hurts IDE experience
- [ ] Fix `include_dir` path resolution — first pattern always fails (dead branch in template registry)

### Bugs
- [ ] `_run_dabs_init` is defined but never called — dead code in CLI
- [ ] `agent_new` name/description logic has a no-op guard
- [ ] `scaffold.rs` ignores `FileRule.overwrite` flag
- [ ] `dabs_schema.rs` has no callers

### Infrastructure
- [ ] Add Python integration tests for `dex agent new`
- [ ] Add Python integration tests for `dex mcp serve`
- [ ] Set up GitHub Pages source to "GitHub Actions" in repo settings (one-time manual step)

## Done

- [x] Initial Rust core — template engine, config, file I/O (`dex-core`)
- [x] PyO3 bindings (`dex-py`)
- [x] Python CLI — `dex init`, `dex agent new`, `dex mcp serve` (`python/dex/`)
- [x] `default` template — plain Python package
- [x] `dabs-package` template — Databricks Asset Bundle
- [x] Multi-variable scaffolding (prompts for all manifest variables)
- [x] MCP server skeleton with `list_templates` implemented
- [x] GitHub Pages via Actions (mkdocs-material docs site)
- [x] Auto-versioning on merge to main (`version.yml` — conventional commits → tag)
- [x] GitHub Releases with platform wheels (`release.yml` — maturin-action)
- [x] `make bump-patch/minor/major` for manual releases
- [x] `/release` skill
- [x] Installation docs defaulting to `uv tool install` from GitHub Releases
