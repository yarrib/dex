---
sha: 31c18f831b0c064458b93d07b802a8c42ef3453e
short_sha: 31c18f8
author: yarrib
date: 2026-03-05
class: [Evolution]
area: Docs, CI & Release
tags: [#evolution]
---

# [Evolution] Feat/documentation (#12)

**Commit:** `31c18f8` · **Author:** yarrib · **Date:** 2026-03-05 · **Area:** Docs, CI & Release

## Summary
                                                            
  - Fix quick install on index/quickstart to use `install.sh` script
  - Add `docs/quickstart.md` — install → scaffold → deploy walkthrough
  - Add `docs/usage/mcp.md` — dex mcp serve reference + Claude wiring
- Add `docs/templates/built-in.md` — all 5 templates with rationale,
variables,
   file trees
- Add `docs/templates/authoring.md` — full template.toml reference +
Jinja2
  guide
  - Add `docs/templates/org-templates.md` — org registry, create_cli(),
  distribution patterns
- Add `docs/extending.md` — org CLI guide: create_cli(), passthroughs,
custom
  commands
  - Move `docs/prd-*.md` to `docs/internal/` (out of public nav)
  - Update mkdocs nav with all new sections
  - Pin `mkdocs>=1.6,<2` to prevent theme-breaking upgrade
  - Add `make docs` / `make docs-serve` targets
  - Add Git Workflow rules to CLAUDE.md
- Update .gitignore: *.so, *.dylib, site/, .coverage, uv.lock,
.claude/plans/
  - Update TASKS.md: mark completed items, fix MCP tool name

  ## Test plan

- [ ] `make docs-serve` — verify all nav sections render with material
theme
  - [ ] Check no broken internal links
  - [ ] Confirm `docs/internal/` PRDs don't appear in nav

---------

Co-authored-by: Claude Sonnet 4.6 <noreply@anthropic.com>

## Changed files

- `.gitignore`
- `CLAUDE.md`
- `TASKS.md`
- `docs/extending.md`
- `docs/index.md`
- `docs/internal/prd-ai-integration.md`
- `docs/internal/prd-templates.md`
- `docs/internal/prd-templating-strategy.md`
- `docs/quickstart.md`
- `docs/templates/authoring.md`
- `docs/templates/built-in.md`
- `docs/templates/org-templates.md`
- `docs/usage/mcp.md`
- `mkdocs.yml`

## Relationships

- **co-occurrence** → [[0e067c2-add-workflow-dispatch-to-docs-deploy-workflow]] (7 shared files)
- **resolved-by** → `#12` _(this commit)_
