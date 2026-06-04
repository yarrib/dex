---
sha: 1f9f2e9837be7cd813ef3df2ab70cdba9ed8cb16
short_sha: 1f9f2e9
author: yarrib
date: 2026-03-05
class: [Dependency]
area: Docs, CI & Release
tags: [#dependency]
---

# [Dependency] chore(release): remove auto-version workflow, add release guide (#13)

**Commit:** `1f9f2e9` · **Author:** yarrib · **Date:** 2026-03-05 · **Area:** Docs, CI & Release

## Summary
   
- Remove `version.yml` — auto-tagger was incompatible with `release.yml`
  version validation                                        
- Add `workflow_dispatch` to `release.yml` for manual re-triggering if
needed
- Add `docs/releasing.md` — full release guide: bump commands, workflow
steps,
  changelog conventions, hotfix flow, failure recovery
  - Document release process in `CLAUDE.md`

  ## Test plan

  - [ ] Verify `version.yml` is gone and no auto-tags fire on merge
- [ ] `make bump-patch` on a clean main creates tag + triggers
`release.yml`
  - [ ] Releasing page appears in docs nav

Co-authored-by: Claude Sonnet 4.6 <noreply@anthropic.com>

## Changed files

- `.github/workflows/release.yml`
- `.github/workflows/version.yml`
- `CLAUDE.md`
- `docs/releasing.md`
- `mkdocs.yml`

## Relationships

- **implemented-in** → [[31c18f8-feat-documentation-12]]
- **co-occurrence** → [[0e067c2-add-workflow-dispatch-to-docs-deploy-workflow]] (4 shared files)
- **co-occurrence** → [[31c18f8-feat-documentation-12]] (2 shared files)
- **resolved-by** → `#13` _(this commit)_
