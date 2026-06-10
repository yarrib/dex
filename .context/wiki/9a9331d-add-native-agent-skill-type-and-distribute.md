---
sha: 9a9331d4a45e4e555be5b92297bacdc0f27d0b70
short_sha: 9a9331d
author: Claude
date: 2026-06-10
class: [Evolution]
area: Skills, Traits & Extensibility
tags: [#evolution]
---

# [Evolution] feat(skills): add native Agent Skill type and distribute project-memory-engine

**Commit:** `9a9331d` · **Author:** Claude · **Date:** 2026-06-10 · **Area:** Skills, Traits & Extensibility

Add a third skill type, `skill`, for native Agent Skills (a folder with
SKILL.md). The installer routes it to `.claude/skills/<name>/SKILL.md`
(and `.ai-skills/skills/<name>/SKILL.md` for generic), preserving the
SKILL.md frontmatter; for Cursor/Copilot the inner frontmatter is
stripped to avoid doubled blocks.

Register a project-memory-engine skill in the default pack so any project
can install it via `dex skills init` and drive `dex context sync`.

## Changed files

- `crates/dex-core/src/skills/installer.rs`
- `crates/dex-core/src/skills/manifest.rs`
- `skills/default/skills.toml`
- `skills/default/skills/project-memory-engine/SKILL.md`

## Relationships

- **influenced-by** → [[6cbcd49-add-prd-for-ai-ready-scaffolding-context-map]] (Skills, Traits & Extensibility)
- **implemented-in** → [[bf31033-add-rust-project-memory-knowledge-graph-engine]]
- **co-occurrence** → [[2f1e593-add-dex-skills-system-agent-skill-pack]] (3 shared files)
