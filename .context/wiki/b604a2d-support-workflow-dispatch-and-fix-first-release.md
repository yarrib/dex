---
sha: b604a2d1996c915bc1f37ffd58ae150ab128f7a1
short_sha: b604a2d
author: yarrib
date: 2026-04-01
class: [Stability]
area: Docs, CI & Release
tags: [#stability]
---

# [Stability] fix(release): support workflow_dispatch and fix first-release changelog

**Commit:** `b604a2d` · **Author:** yarrib · **Date:** 2026-04-01 · **Area:** Docs, CI & Release

## Summary

- Add `prepare` job to resolve tag/version from both tag-push and
`workflow_dispatch` events (`GITHUB_REF_NAME` is a branch name for
manual triggers, which broke `validate` and `release` jobs)
- Add `version` input to `workflow_dispatch` so releases can be
triggered from the GitHub Actions UI without pushing a tag manually
- Switch `git-cliff` from `--latest` to `--unreleased --tag <tag>` so
the first-release changelog shows the correct version label instead of
"Unreleased"
- Pass explicit `tag_name` and `name` to `softprops/action-gh-release`
for manual dispatch

## Test plan

- [ ] Merge this PR
- [ ] Go to Actions → Release → Run workflow → enter `0.1.1` to trigger
the first release
- [ ] Verify GitHub Release is created with 4 platform binaries attached
- [ ] Run `curl -sSf
https://raw.githubusercontent.com/yarrib/dex/main/install.sh | sh` —
should install successfully
- [ ] Check docs changelog page shows v0.1.1 entries

https://claude.ai/code/session_01FJhH5ED2UEauXPZgA8WatD

## Changed files

_No tracked source files (vendored/lock changes only)._

## Relationships

- **modified-by** → [[e3efd53-rewrite-all-docs-for-rust-binary-architecture-25]]
