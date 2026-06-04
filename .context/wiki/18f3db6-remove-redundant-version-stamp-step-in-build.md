---
sha: 18f3db6176ef39f9146f4bbbe263273f3c66a2fd
short_sha: 18f3db6
author: yarrib
date: 2026-03-07
class: [Stability]
area: Docs, CI & Release
tags: [#stability]
---

# [Stability] fix(release): remove redundant version stamp step in build jobs

**Commit:** `18f3db6` · **Author:** yarrib · **Date:** 2026-03-07 · **Area:** Docs, CI & Release

Version is already committed in pyproject.toml as part of the bump PR.
Stamping from the tag is unnecessary and broke when the tag pointed to
a commit before the bump-version.py idempotency fix.

Co-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>

## Changed files

- `.github/workflows/release.yml`

## Relationships

- **modified-by** → [[31c18f8-feat-documentation-12]]
- **implemented-in** → [[31c18f8-feat-documentation-12]]
- **co-occurrence** → [[b5d760e-fix-bump-version-idempotency-and-replace-git]] (1 shared file)
- **co-occurrence** → [[1f9f2e9-remove-auto-version-workflow-add-release-guide]] (1 shared file)
- **co-occurrence** → [[0e067c2-add-workflow-dispatch-to-docs-deploy-workflow]] (1 shared file)
