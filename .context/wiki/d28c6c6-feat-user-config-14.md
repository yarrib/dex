---
sha: d28c6c63b78380cf8da615c317eedbe1aa83cc99
short_sha: d28c6c6
author: yarrib
date: 2026-03-06
class: [Evolution]
area: Foundation & Architecture
tags: [#evolution]
---

# [Evolution] Feat/user config (#14)

**Commit:** `d28c6c6` · **Author:** yarrib · **Date:** 2026-03-06 · **Area:** Foundation & Architecture

Summary

  - Adds python/dex/config.py — loads ~/.config/dex/config.toml (user)
  and ./dex.toml (project), merges them with project taking precedence.
  Supports git remote template sources (SSH + HTTPS) that clone/pull to
  ~/.cache/dex/templates/<name>/.
  - Updates cli.py — _collect_templates() builds a unified registry
  across embedded templates, config-sourced dirs/remotes, and the
  extra_dir from create_cli(). init_command uses @click.pass_context to
  read templates_dir from the root group.
  - Exports DexConfig, RemoteSource, load_config from ext.py for use in
  org CLIs.
  - Fixes a Rust bug in registry.rs where include_dir 0.7 path semantics
  caused list_embedded_templates() to always return []. File paths in
  embedded dirs are root-relative, so dir.get_file("template.toml")
  always missed — fixed to
  dir.get_file(dir.path().join("template.toml")). Adds a Rust regression
  test.
  - Migrates from [project.optional-dependencies] to [dependency-groups]
  (PEP 735) across pyproject.toml, ci.yml, docs.yml, and Makefile.
- Fixes several make targets for macOS / Apple Silicon: cargo clippy -p
   dex-core and cargo test -p dex-core to avoid PyO3 linker errors
  outside maturin; uv run maturin develop --skip-install to avoid
  maturin's broken uv pip install --group invocation.

  Test plan

  - make test passes (20 Rust unit tests, 64 Python tests, 83% coverage)
  - dex init --template default --no-prompt scaffolds a project
  - Config parsing tests cover: empty file, missing file, templates_dir,
  HTTPS remote, SSH remote, multiple remotes, malformed remotes, merge
  precedence, additive remotes, deduplication
  - _collect_templates returns embedded templates when no config is
  present
  - _collect_templates with extra_dir includes local templates alongside
  embedded ones

---------

## Changed files

- `.github/workflows/ci.yml`
- `.github/workflows/docs.yml`
- `CLAUDE.md`
- `Makefile`
- `TASKS.md`
- `crates/dex-core/src/template/registry.rs`
- `docs/releasing.md`
- `pyproject.toml`
- `python/dex/agent.py`
- `python/dex/cli.py`
- `python/dex/config.py`
- `python/dex/ext.py`
- `tests/test_cli.py`
- `tests/test_config.py`

## Relationships

- **implemented-in** → [[31c18f8-feat-documentation-12]]
- **co-occurrence** → [[31c18f8-feat-documentation-12]] (2 shared files)
- **resolved-by** → `#14` _(this commit)_
