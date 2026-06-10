---
sha: 4535cd05e9d32a3228f3a0ef450e72471b2e5634
short_sha: 4535cd0
author: yarrib
date: 2026-04-02
class: [Evolution]
area: Docs, CI & Release
tags: [#evolution]
---

# [Evolution] feat(devcontainer): ai-dev-kit integration with profile-based skill setup (#38)

**Commit:** `4535cd0` · **Author:** yarrib · **Date:** 2026-04-02 · **Area:** Docs, CI & Release

## Summary

- Adds `.devcontainer/` with Rust + Python 3.12 image and
`postCreateCommand`
- Adds `scripts/setup_dev_kit.sh` — installs [Databricks AI Dev
Kit](https://github.com/databricks-solutions/ai-dev-kit) with
profile-based skill selection, then vendors skills into
`skills/databricks/` so `dex skills sync` can distribute them
- Adds `.devcontainer/config.toml` for org-level preset (commit to share
with team); falls back to interactive prompt or non-interactive default

**Profiles** map directly to ai-dev-kit's own profiles:
| Profile | Skills |
|---|---|
| `ai-ml-engineer` | agents, MLflow, vector search, model serving |
| `data-engineer` | pipelines, DLT, Unity Catalog, Iceberg |
| `analyst` | AI/BI dashboards, Genie, SQL |
| `app-developer` | Databricks Apps, FastAPI, Streamlit |

**Assistants:** `claude` \| `cursor` \| `copilot` \| `codex` \| `gemini`
\| `all`

## Design notes

`setup_dev_kit.sh` treats ai-dev-kit as a *source* for the dex skills
system — vendoring installed skills into `skills/databricks/` rather
than running as a parallel install path. After setup, `dex skills sync`
handles distribution to the assistant's config directory.

## Test plan

- [ ] Open in GitHub Codespaces — verify `postCreateCommand` completes
and skills land in `.claude/skills/`
- [ ] Set `config.toml` to `data-engineer` / `cursor` — verify correct
profile installs
- [ ] Delete `config.toml` `[ai]` block — verify interactive prompt
appears on TTY, default used when non-interactive
- [ ] Run `dex skills sync` after setup — verify skills are distributed
correctly

## Changed files

- `.devcontainer/config.toml`
- `.devcontainer/devcontainer.json`
- `.devcontainer/setup.sh`
- `scripts/setup_dev_kit.sh`

## Relationships

- **influenced-by** → [[e3efd53-rewrite-all-docs-for-rust-binary-architecture-25]] (Docs, CI & Release)
- **implemented-in** → [[2f1e593-add-dex-skills-system-agent-skill-pack]]
- **resolved-by** → `#38` _(this commit)_
