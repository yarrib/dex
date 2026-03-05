# dex — Task Backlog

## In Progress

_(nothing active)_

## Backlog

### Features
- [ ] Fix `init_command` variable prompting — currently only `project_name` is prompted; other manifest variables are skipped
- [ ] Implement MCP server tool stubs — `scaffold_project`, `scaffold_agent`, `get_template_variables` are currently unimplemented in `mcp_server.py`
- [x] Add `dabs-etl` template (DLT pipeline, autoloader source, Unity Catalog target)
- [x] Add `dabs-ml` template (MLflow experiment, model registry, training job, serving endpoint)
- [x] Add `dabs-aiagent` template (Mosaic AI agent, vector search index, model serving endpoint)
- [ ] Add `dabs-dashboard` template stub — Databricks Lakeview dashboard definition (`resources/<name>_dashboard.yml`, YAML schema, deploy instructions)
- [ ] Add `dabs-genie-space` template stub — Databricks Genie space definition (`resources/<name>_genie_space.yml`, curated SQL instructions file, space config)
- [ ] Add `databricks-app-streamlit` template stub — Databricks Apps + Streamlit (`app.py`, `app.yaml`, `requirements.txt`, deploy via `databricks apps deploy`)
- [ ] Add `databricks-app-react` template stub — Databricks Apps + React (`package.json`, Vite config, `src/App.tsx`, `app.yaml`, build + deploy instructions)
- [x] `_core.pyi` stub file — type stubs exist in `python/dex/_core.pyi`
- [ ] Fix `include_dir` path resolution — first pattern always fails (dead branch in template registry)
- [ ] Wire user template directories through Python → PyO3 → Rust (`TemplateSource::Directory` exists; `create_cli()` ignores `templates_dir`)
- [ ] Implement user config loading from `~/.config/dex/config.toml` (Python layer, per arch rules)

### Documentation
- [x] Write quickstart guide (`docs/quickstart.md`) — install → `dex init` → inspect project
- [x] Write template authoring guide (`docs/templates/authoring.md`) — manifest format, minijinja syntax, file rules
- [x] Write org-CLI / extending guide (`docs/extending.md`) — `create_cli()`, custom templates, passthrough commands
- [x] Document built-in templates (`docs/templates/built-in.md`) — rationale for each default pattern (`default`, `dabs-package`, etc.), design decisions, when to use each
- [x] Document org template pattern (`docs/templates/org-templates.md`) — how to publish a private template registry, wire it into a `create_cli()` org-CLI, and distribute via internal PyPI or direct Git URL ("pull and attach")
- [ ] Move `docs/prd-*.md` to `docs/internal/` so they don't appear in public nav
- [x] Add MCP integration guide (`docs/usage/mcp.md` — Claude Desktop + Claude Code wiring)
- [x] Update mkdocs nav to add Quickstart, Templates section, and Extending page

### Bugs
- [x] `_run_dabs_init` is defined but never called — removed; DABs composite flow TBD
- [x] `agent_new` name/description logic has a no-op guard
- [x] `scaffold.rs` ignores `FileRule.overwrite` flag — already implemented (lines 61-64)
- [x] `dabs_schema.rs` has no callers — module is valid and tested; will be wired in DABs composite flow
- [x] `PassthroughCommand` extends deprecated `click.BaseCommand` — migrated to `click.Command`

### Infrastructure
- [x] Add `ci.yml` — PR/push CI for Python (ty, ruff, pytest) and Rust (clippy, fmt, test)
- [x] Add `install.sh` — curl-pipeable install from GitHub Releases
- [x] Configure mike for versioned docs (Pages source set to "Deploy from branch: gh-pages"; branch protection on main; workflow_dispatch added to docs.yml)
- [x] Improve `release.yml` — tag validation + changelog via git-cliff
- [x] Add Python integration tests for `dex agent new`
- [x] Add Python integration tests for `dex mcp serve`
- [x] Add pre-commit config (`.pre-commit-config.yaml`) — ruff + cargo-clippy hooks
- [x] Add coverage threshold to pytest (`--cov=dex --cov-fail-under=80`) + `pytest-cov` dep
- [x] Add `bandit` security scan to Python CI step

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
- [x] Fix slugifier for hyphenated package names — `table-anomaly-monitor` → `table_anomaly_monitor`
- [x] Expose `system_prompt` and `claude_md` in `AgentScaffoldResultPy` PyO3 binding
