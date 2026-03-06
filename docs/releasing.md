# Releasing

Releases are manual and tag-driven. Because `main` is protected, version bumps go through a PR. The tag is pushed after the PR is merged, which triggers the release workflow.

## Prerequisites

- All changes for the release are merged to `main`
- You have push access to the repo (to push tags)

## Release flow

### 1. Decide the version bump

Follow [Semantic Versioning](https://semver.org/):

| Change type | Command |
|---|---|
| Bug fixes, docs, chores | `make bump-patch` → `0.1.0` → `0.1.1` |
| New features, backwards-compatible | `make bump-minor` → `0.1.0` → `0.2.0` |
| Breaking changes | `make bump-major` → `0.1.0` → `1.0.0` |

### 2. Create the version bump PR

```bash
git checkout main && git pull
git checkout -b chore/release-v0.x.y
make bump-patch   # or bump-minor / bump-major
git push -u origin chore/release-v0.x.y
```

`make bump-patch` will:

1. Update the version in `pyproject.toml` and both `Cargo.toml` files
2. Commit the changes with message `chore: bump version to vX.Y.Z`

Open a PR for the branch, get it merged.

!!! warning "Working tree must be clean"
    The bump commands check for uncommitted changes and will abort if any exist.

### 3. Tag and push

After the PR is merged:

```bash
git checkout main && git pull
make tag-release
```

`make tag-release` tags the current `HEAD` with the version in `pyproject.toml` and pushes the tag to GitHub. It also guards against running on the wrong branch.

### 4. Watch the release workflow

Go to **Actions → Release** on GitHub. The workflow:

1. Validates the tag format (`v<major>.<minor>.<patch>`)
2. Verifies `pyproject.toml` version matches the tag
3. Generates a changelog from conventional commits (git-cliff)
4. Builds wheels for Linux x86\_64, macOS Apple Silicon, macOS Intel
5. Builds an sdist
6. Creates a GitHub Release with all artifacts attached

### 5. Verify the release

- Check [GitHub Releases](https://github.com/yarrib/dex/releases) for the new release
- Confirm wheels are attached for all platforms
- Confirm the changelog looks correct
- The docs site will auto-deploy the new versioned docs (e.g. `0.2`) via the `docs.yml` workflow

---

## Commit conventions and changelog

The changelog is generated automatically from commit messages using [git-cliff](https://git-cliff.org/). Use conventional commit prefixes so changes appear correctly:

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

## Hotfix releases

Same flow as a regular release — fix on a branch, PR to main, then tag:

```bash
git checkout -b fix/critical-bug
# make your fix
git push -u origin fix/critical-bug
# open PR, merge
git checkout main && git pull
make bump-patch
git push -u origin chore/release-vX.Y.Z
# open PR, merge
git checkout main && git pull
make tag-release
```

There is no separate hotfix branch — all releases go through main.

---

## If the release workflow fails

The most common causes:

- **Tag/version mismatch** — `pyproject.toml` version doesn't match the tag. Delete the tag, fix the version, re-tag.
- **Build failure** — a Rust compilation error in the wheel build. Fix the code, delete the tag, re-tag.

To delete a tag and re-release:

```bash
git tag -d v0.1.1
git push origin :refs/tags/v0.1.1
# fix the issue, then
make tag-release
```
