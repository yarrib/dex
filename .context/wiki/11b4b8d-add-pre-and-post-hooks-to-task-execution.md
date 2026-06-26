---
sha: 11b4b8d5f2c069b61db12c3c5ea59527759e2433
short_sha: 11b4b8d
author: Claude
date: 2026-06-26
class: [Evolution]
area: Foundation & Architecture
tags: [#evolution]
---

# [Evolution] feat(run): add pre and post hooks to task execution

**Commit:** `11b4b8d` · **Author:** Claude · **Date:** 2026-06-26 · **Area:** Foundation & Architecture

Tasks in dex.toml can now declare `pre` and `post` arrays of shell
commands. Pre-hooks run before the task command; a failure aborts
the task. Post-hooks run after the task succeeds.

Example:
  [tasks.deploy]
  command = "databricks bundle deploy"
  pre = ["./scripts/check-auth.sh"]
  post = ["./scripts/notify-team.sh"]

Claude-Session: https://claude.ai/code/session_01DUdDxDVTJj5eyVTvDzGR6w

## Changed files

- `crates/dex-cli/src/commands/run.rs`
- `crates/dex-core/src/config.rs`
- `docs/SPEC.md`

## Relationships

- **influenced-by** → [[ff57dbb-add-runnable-org-setup-examples-and-fix]] (Foundation & Architecture)
- **co-occurrence** → [[2f1e593-add-dex-skills-system-agent-skill-pack]] (3 shared files)
- **co-occurrence** → [[6cbcd49-add-prd-for-ai-ready-scaffolding-context-map]] (2 shared files)
- **co-occurrence** → [[bacd088-add-dex-run-task-command-28]] (2 shared files)
