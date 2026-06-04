---
sha: 7b9ade03d7be3d2f6391fa2cb69712decebd861b
short_sha: 7b9ade0
author: yarrib
date: 2026-03-09
class: [Dependency]
area: Foundation & Architecture
tags: [#dependency]
---

# [Dependency] chore: release v0.1.0 (#17)

**Commit:** `7b9ade0` · **Author:** yarrib · **Date:** 2026-03-09 · **Area:** Foundation & Architecture

## Summary

- Reset version to `v0.1.0` across `pyproject.toml`, `Cargo.toml` files,
and `Cargo.lock`
- Updated `README.md`, `docs/index.md`, and `docs/installation.md` to
make the uv-backed installer script the primary install path
- Removed unused `workflow_dispatch` tag input from `release.yml`

## Test plan

- [ ] Merge to main
- [ ] Run `make tag-release` to tag `v0.1.0` and trigger the release
workflow
- [ ] Verify wheels are built and attached to the GitHub Release
- [ ] Test `curl -sSf
https://raw.githubusercontent.com/yarrib/dex/main/install.sh | sh` in a
clean environment

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-authored-by: Claude Sonnet 4.6 <noreply@anthropic.com>

## Changed files

- `.github/workflows/release.yml`
- `README.md`
- `crates/dex-core/Cargo.toml`
- `crates/dex-py/Cargo.toml`
- `docs/index.md`
- `docs/installation.md`
- `pyproject.toml`

## Relationships

- **implemented-in** → [[31c18f8-feat-documentation-12]]
- **co-occurrence** → [[0e067c2-add-workflow-dispatch-to-docs-deploy-workflow]] (7 shared files)
- **co-occurrence** → [[3eb93ae-chore-release-v0-1-4-16]] (3 shared files)
- **co-occurrence** → [[00b3712-chore-release-v0-1-3-15]] (3 shared files)
- **resolved-by** → `#17` _(this commit)_
