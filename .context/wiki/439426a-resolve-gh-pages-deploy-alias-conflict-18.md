---
sha: 439426a45e6e0aa4ef07bd327f4741138ba765eb
short_sha: 439426a
author: yarrib
date: 2026-03-09
class: [Stability]
area: Docs, CI & Release
tags: [#stability]
---

# [Stability] fix(docs): resolve gh-pages deploy alias conflict (#18)

**Commit:** `439426a` · **Author:** yarrib · **Date:** 2026-03-09 · **Area:** Docs, CI & Release

## Summary

- Change main branch docs deploy from `mike deploy latest` to `mike
deploy dev latest`
- This ensures `latest` is always an *alias* (never a standalone
version), so the tag-based deploy step can reassign it without conflict

## Root cause

Previous runs created a version *named* `latest` on push to main. The
tag deploy step then tried to use `latest` as an alias for the versioned
release — mike rejected it because a version and alias can't share the
same name.

## Also needed (GitHub settings)

The `gh-pages` branch is protected and blocking the Actions bot from
pushing. Go to **Settings → Rules** and add GitHub Actions as a bypass
actor for the `gh-pages` rule.

## Changed files

- `.github/workflows/docs.yml`

## Relationships

- **modified-by** → [[d28c6c6-feat-user-config-14]]
- **implemented-in** → [[31c18f8-feat-documentation-12]]
- **co-occurrence** → [[d28c6c6-feat-user-config-14]] (1 shared file)
- **resolved-by** → `#18` _(this commit)_
