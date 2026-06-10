---
sha: bb227029828d164b138a89b1e1f7b4924af6373e
short_sha: bb22702
author: yarrib
date: 2026-04-01
class: [Stability]
area: Docs, CI & Release
tags: [#stability]
---

# [Stability] fix(release): use cross for Linux musl targets

**Commit:** `bb22702` · **Author:** yarrib · **Date:** 2026-04-01 · **Area:** Docs, CI & Release

## Summary

- Replace manual `musl-tools` + `gcc-aarch64-linux-gnu` linker setup
with `cross` for both Linux targets
- Root cause: `gcc-aarch64-linux-gnu` targets the glibc ABI and cannot
link musl binaries
- `cross` uses Docker containers with correct musl toolchains
pre-installed for both `x86_64` and `aarch64`
- macOS targets unchanged (native `cargo` build works for both
architectures)

## Changes

- Add `build_cmd: cross | cargo` to matrix so a single build step
handles both cases
- Remove per-target musl-tools apt install and linker config steps
- Add `cargo install cross --locked` step for Linux targets

## After merging

Delete and re-push the `v0.1.1` tag, or trigger via **Actions → Release
→ Run workflow → `0.1.1`**.

## Changed files

- `.github/workflows/release.yml`

## Relationships

- **modified-by** → [[e3efd53-rewrite-all-docs-for-rust-binary-architecture-25]]
- **implemented-in** → [[bacd088-add-dex-run-task-command-28]]
- **co-occurrence** → [[5da5907-support-workflow-dispatch-and-fix-first-release]] (1 shared file)
- **co-occurrence** → [[a378387-align-install-sh-artifact-names-with-release]] (1 shared file)
- **co-occurrence** → [[18f3db6-remove-redundant-version-stamp-step-in-build]] (1 shared file)
