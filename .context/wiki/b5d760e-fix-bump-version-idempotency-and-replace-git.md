---
sha: b5d760e8b5a39c6edf6c03a7e23fcd4c285e826d
short_sha: b5d760e
author: yarrib
date: 2026-03-06
class: [Stability]
area: Docs, CI & Release
tags: [#stability]
---

# [Stability] fix(release): fix bump-version idempotency and replace git-cliff Docker action

**Commit:** `b5d760e` · **Author:** yarrib · **Date:** 2026-03-06 · **Area:** Docs, CI & Release

- bump-version.py: check pattern exists before substitution so setting
  the same version twice doesn't error (fixes CI stamp step on pre-bumped repos)
- release.yml: replace orhun/git-cliff-action (Docker, fragile) with
  taiki-e/install-action + direct git-cliff invocation

## Changed files

- `.github/workflows/release.yml`
- `scripts/bump-version.py`

## Relationships

- **modified-by** → [[31c18f8-feat-documentation-12]]
- **implemented-in** → [[31c18f8-feat-documentation-12]]
