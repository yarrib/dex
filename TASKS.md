# dex — Task Backlog

## In Progress

_(nothing active)_

## Backlog

### Features
- [ ] Fix `init_command` variable prompting — currently only `project_name` is prompted; other manifest variables are skipped
- [ ] Implement MCP server tool stubs — `scaffold_project`, `scaffold_agent`, `get_template` are currently unimplemented in `mcp_server.py`
- [ ] Add `dabs-etl` template (DLT pipeline, autoloader source, Unity Catalog target)
- [ ] Add `dabs-ml` template (MLflow experiment, model registry, training job, serving endpoint)
- [ ] Add `dabs-aiagent` template (Mosaic AI agent, vector search index, model serving endpoint)
- [ ] `_core.pyi` stub file — no type stubs exist for the PyO3 extension, hurts IDE experience
- [ ] Fix `include_dir` path resolution — first pattern always fails (dead branch in template registry)
- [ ] Wire user template directories through Python → PyO3 → Rust (`TemplateSource::Directory` exists; `create_cli()` ignores `templates_dir`)
- [ ] Implement user config loading from `~/.config/dex/config.toml` (Python layer, per arch rules)

### Documentation
- [ ] Write quickstart guide (`docs/quickstart.md`) — install → `dex init` → inspect project
- [ ] Write template authoring guide (`docs/templates/authoring.md`) — manifest format, minijinja syntax, file rules
- [ ] Write org-CLI / extending guide (`docs/extending.md`) — `create_cli()`, custom templates, passthrough commands
- [ ] Document built-in templates (`docs/templates/built-in.md`) — rationale for each default pattern (`default`, `dabs-package`, etc.), design decisions, when to use each
- [ ] Document org template pattern (`docs/templates/org-templates.md`) — how to publish a private template registry, wire it into a `create_cli()` org-CLI, and distribute via internal PyPI or direct Git URL ("pull and attach")
- [ ] Move `docs/prd-*.md` to `docs/internal/` so they don't appear in public nav
- [ ] Add MCP integration guide (how to wire dex MCP server into Claude Desktop / other AI tools)
- [ ] Update mkdocs nav to add Quickstart, Templates section, and Extending page

### Bugs
- [ ] `_run_dabs_init` is defined but never called — dead code in CLI
- [ ] `agent_new` name/description logic has a no-op guard
- [ ] `scaffold.rs` ignores `FileRule.overwrite` flag
- [ ] `dabs_schema.rs` has no callers

### Infrastructure
- [x] Add `ci.yml` — PR/push CI for Python (ty, ruff, pytest) and Rust (clippy, fmt, test)
- [x] Add `install.sh` — curl-pipeable install from GitHub Releases
- [x] Configure mike for versioned docs (one-time manual step: change Pages source from "GitHub Actions" artifact → "Deploy from branch: gh-pages")
- [x] Improve `release.yml` — tag validation + changelog via git-cliff
- [ ] Add Python integration tests for `dex agent new`
- [ ] Add Python integration tests for `dex mcp serve`
- [ ] Add pre-commit config (`.pre-commit-config.yaml`) — ruff + cargo-clippy hooks
- [ ] Add coverage threshold to pytest (`--cov=dex --cov-fail-under=80`) + `pytest-cov` dep
- [ ] Add `bandit` security scan to Python CI step

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
