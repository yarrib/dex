---
name: release
description: Manage versioning and releases for dex
---

Manage versioning and releases for dex.

Releases are **tag-driven**: a `vX.Y.Z` tag on `main` triggers the `release.yml`
workflow, which validates the tag, generates a changelog, builds the native
binaries (no wheels — dex ships a single Rust binary), and publishes a GitHub
Release. Because `main` is protected, the version bump goes through a PR first.
You don't tag by hand — merging the bump PR auto-tags and releases.

## 1. Check the current version

```bash
make version            # or: bash scripts/bump-version.sh
```

## 2. Open a release PR

From an up-to-date `main`:

```bash
git checkout main && git pull

make bump-patch         # 0.2.0 → 0.2.1  (fix:/chore:/refactor:/docs:/test:)
make bump-minor         # 0.2.0 → 0.3.0  (feat:)
make bump-major         # 0.2.0 → 1.0.0  (BREAKING CHANGE)
```

Each `bump-*` target runs `scripts/release.sh`, which branches `chore/release-vX.Y.Z`,
bumps the version in `crates/dex-core/Cargo.toml` and `crates/dex-cli/Cargo.toml`
via `scripts/bump-version.sh`, commits `chore: bump version to vX.Y.Z`, pushes the
branch, and opens a PR with `gh`. It aborts if the working tree is dirty or you're
not on `main`. (No `gh`? It still pushes the branch — open the PR manually.)

## 3. Merge the PR — tagging is automatic

When the PR merges to `main`, `.github/workflows/tag-on-merge.yml` runs: it reads
the version from `crates/dex-cli/Cargo.toml`, and if no `vX.Y.Z` tag exists yet it
pushes the tag and dispatches `release.yml`. That's it — the release ships.

Why dispatch rather than rely on the tag push? A tag pushed by the built-in
`GITHUB_TOKEN` does **not** trigger other workflows (GitHub's loop-prevention
rule), so `release.yml`'s `on: push: tags` would never fire. The merge workflow
pushes the tag (so the changelog/release reference it) **and** calls
`gh workflow run release.yml`, which the built-in token is allowed to do.
No PAT required.

### GitHub-side setup (one time)

- **Settings → Actions → General → Workflow permissions** → select
  **Read and write permissions**. This lets the built-in `GITHUB_TOKEN` push the
  tag and dispatch `release.yml`. (`tag-on-merge.yml` also requests
  `contents: write` and `actions: write` explicitly, but the repo setting must
  permit it.)
- Nothing else — no personal access token, no extra secrets.
- If org policy forbids `workflow_dispatch` from the built-in token, add a
  fine-grained PAT (Contents: read & write) as a secret and pass it as the
  `token:` on the checkout step in `tag-on-merge.yml`; a plain tag push then
  triggers `release.yml` directly (drop the `gh workflow run` line).

### Manual fallback

You can still drive a release by hand:

```bash
git checkout main && git pull
make tag-release        # tags main HEAD with the Cargo.toml version and pushes
```

Or trigger the workflow from **Actions → Release → Run workflow**
(`workflow_dispatch`) with the version (e.g. `0.2.1`). Both validate that the
version matches `crates/dex-cli/Cargo.toml` on `main`, so the bump PR must be
merged first either way.

## 4. What `release.yml` does

1. Resolves the tag/version (from the pushed tag or the `workflow_dispatch` input)
2. Validates the tag format `v<major>.<minor>.<patch>` and that it matches
   `crates/dex-cli/Cargo.toml`
3. Generates a changelog from conventional commits with `git-cliff`
4. Builds native binaries: `dex-linux-x86_64`, `dex-linux-aarch64` (musl, via
   `cross`), `dex-macos-x86_64`, `dex-macos-aarch64`
5. Creates the GitHub Release **as a draft** (binaries + changelog) — review it
   and click **Publish release** to make it public

## Troubleshooting

- **`bump-*` fails "working tree is dirty"** — commit or stash changes first.
- **`bump-*` fails "run from main"** — `git checkout main && git pull`, then re-run.
- **Merged the PR but no release ran** — check that **Workflow permissions** is set
  to *Read and write* (see GitHub-side setup); otherwise `tag-on-merge.yml` can't
  push the tag or dispatch `release.yml`.
- **`tag-on-merge.yml` did nothing** — the `vX.Y.Z` tag already exists (the merge
  didn't change the version). Bump the version in a new PR.
- **Release workflow fails at "Verify Cargo.toml version matches tag"** — the tag
  doesn't match `crates/dex-cli/Cargo.toml` on the tagged commit. Bump + merge
  first, then re-tag.
- **Tag already exists / want to redo** — delete it locally and remotely, then
  re-tag: `git tag -d vX.Y.Z && git push origin :refs/tags/vX.Y.Z`
