# Releasing

Releases are **tag-driven and automated**. Because `main` is protected, the
version bump goes through a PR. Merging that PR tags the release and kicks off the
build automatically — you don't push tags by hand.

## Prerequisites

- All changes for the release are merged to `main`
- You can open and merge PRs

## Release flow

### 1. Decide the version bump

Follow [Semantic Versioning](https://semver.org/):

| Change type | Command |
|---|---|
| Bug fixes, docs, chores | `make bump-patch` → `0.1.0` → `0.1.1` |
| New features, backwards-compatible | `make bump-minor` → `0.1.0` → `0.2.0` |
| Breaking changes | `make bump-major` → `0.1.0` → `1.0.0` |

### 2. Open the release PR

From an up-to-date `main`:

```bash
git checkout main && git pull
make bump-patch   # or bump-minor / bump-major
```

`make bump-patch` runs `scripts/release.sh`, which:

1. Branches `chore/release-vX.Y.Z`
2. Bumps the version in the `Cargo.toml` files
3. Commits `chore: bump version to vX.Y.Z`
4. Pushes the branch and opens a PR with `gh`

> **Note:** It aborts if the working tree is dirty or you're not on `main`. Without
> the `gh` CLI it still pushes the branch — open the PR manually.

CI runs `make check-version`, which fails if the version is behind the latest tag
(that would make the release a no-op).

### 3. Merge the PR — tagging and release are automatic

When the PR merges to `main`, `.github/workflows/tag-on-merge.yml`:

1. Reads the version from `crates/dex-cli/Cargo.toml`
2. If no `vX.Y.Z` tag exists yet, pushes the tag
3. Dispatches the `Release` workflow

A merge that doesn't change the version releases nothing — that's intended.

### 4. Watch the release workflow

Go to **Actions → Release** on GitHub. The workflow:

1. Validates the tag format (`v<major>.<minor>.<patch>`) and that it matches `Cargo.toml`
2. Generates a changelog from conventional commits (git-cliff)
3. Builds native binaries:
   - Linux x86\_64 (musl)
   - Linux aarch64 (musl)
   - macOS Apple Silicon
   - macOS Intel
4. Creates the GitHub Release **as a draft** with the binaries attached

### 5. Review the draft, then publish

The release is created as a **draft** — built and tagged automatically, but not
yet public. This is the human checkpoint:

- Open the draft under [Releases](https://github.com/yarrib/dex/releases)
- Confirm binaries are attached for all platforms and the changelog looks right
- Click **Publish release** to make it public

The docs site redeploys automatically right after the tag is pushed (see below),
so the changelog updates regardless of whether you've published the draft yet.

---

## Changelog and the docs site

The changelog is generated from commit messages with
[git-cliff](https://git-cliff.org/) — never hand-edited. `docs/changelog.md` is a
build artifact (gitignored); `docs.yml` regenerates it on each deploy.

The docs site redeploys:

- on every push to `main` (keeps the **Unreleased** section current), and
- on release — `tag-on-merge.yml` dispatches `docs.yml` right after pushing the
  tag, so the just-released commits move under their version heading. (A tag or a
  `GITHUB_TOKEN`-dispatched run can't trigger workflows on its own, so the deploy
  is dispatched explicitly rather than chained off the tag or the Release run.)

Use conventional commit prefixes so changes appear in the right section:

| Prefix | Changelog section |
|---|---|
| `feat:` | Features |
| `fix:` | Bug Fixes |
| `refactor:` | Refactoring |
| `docs:` | Documentation |
| `test:` | Testing |
| `chore:` | Chores |
| `perf:` | Performance |

Commits without a conventional prefix are filtered out of the changelog.

---

## Manual fallback

The automated path covers normal releases. If you need to drive one by hand
(e.g. the merge workflow is disabled, or re-cutting a botched tag):

```bash
git checkout main && git pull
make tag-release        # tags main HEAD with the Cargo.toml version and pushes
```

Or use **Actions → Release → Run workflow** (`workflow_dispatch`) with the version
(e.g. `0.2.3`). Both validate the version against `Cargo.toml` on `main`, so the
bump PR must be merged first either way.

---

## Hotfix releases

Same automated flow — fix on a branch, merge, then bump:

```bash
git checkout -b fix/critical-bug
# make your fix
git push -u origin fix/critical-bug
# open PR, merge

git checkout main && git pull
make bump-patch         # opens the release PR; merging it ships the hotfix
```

---

## If the release workflow fails

The most common causes:

- **Tag/version mismatch** — `Cargo.toml` version doesn't match the tag. Fix the
  version on `main`, delete the tag, re-trigger.
- **Build failure** — a Rust compilation error. Fix the code and re-release.

To delete a tag and redo:

```bash
git tag -d v0.1.1
git push origin :refs/tags/v0.1.1
# fix the issue, then re-run the release (bump PR, or make tag-release)
```
