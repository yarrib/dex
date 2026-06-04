---
sha: 1078c43543b5b791d78cdb59222e17c35463410e
short_sha: 1078c43
author: yarrib
date: 2026-04-02
class: [Dependency]
area: Foundation & Architecture
tags: [#dependency]
---

# [Dependency] chore: bump version to v0.2.0 (#39)

**Commit:** `1078c43` · **Author:** yarrib · **Date:** 2026-04-02 · **Area:** Foundation & Architecture

Bumps version from `0.1.1` → `0.2.0`.

## Changes since v0.1.1

- `feat(devcontainer)`: ai-dev-kit integration with profile-based skill
setup
- `feat(mcp)`: implement scaffold_agent tool and add .mcp.json
- `feat(skills)`: add dex skills system — agent skill pack management

Minor bump per [conventional
commits](https://www.conventionalcommits.org/) — multiple `feat:`
merges.

## After merging

```bash
git checkout main && git pull
make tag-release
```

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Changed files

- `crates/dex-cli/Cargo.toml`
- `crates/dex-core/Cargo.toml`

## Relationships

- **implemented-in** → [[2f1e593-add-dex-skills-system-agent-skill-pack]]
- **co-occurrence** → [[a378387-align-install-sh-artifact-names-with-release]] (2 shared files)
- **co-occurrence** → [[30690b9-port-dex-to-pure-rust-single-binary-21]] (2 shared files)
- **co-occurrence** → [[bf5f8b5-add-regression-tests-for-embedded-template]] (1 shared file)
- **resolved-by** → `#39` _(this commit)_
