# Releasing

Releases are manual and tag-driven. A maintainer bumps the version locally, pushes the tag, and GitHub Actions builds and publishes the release.

## Prerequisites

- You are on `main` with a clean working tree
- All changes for the release are merged
- You have push access to the repo (to push tags)

## Release flow

### 1. Decide the version bump

Follow [Semantic Versioning](https://semver.org/):

| Change type | Command |
|---|---|
| Bug fixes, docs, chores | `make bump-patch` → `0.1.0` → `0.1.1` |
| New features, backwards-compatible | `make bump-minor` → `0.1.0` → `0.2.0` |
| Breaking changes | `make bump-major` → `0.1.0` → `1.0.0` |

### 2. Run the bump command

```bash
git checkout main
git pull
make bump-patch   # or bump-minor / bump-major
```

This will:

1. Update the version in `pyproject.toml` and both `Cargo.toml` files
2. Create a git tag (e.g. `v0.1.1`)
3. Push the tag to GitHub

!!! warning "Working tree must be clean"
    The bump commands check for uncommitted changes and will abort if any exist. Commit or stash everything first.

### 3. Watch the release workflow

Go to **Actions → Release** on GitHub. The workflow:

1. Validates the tag format (`v<major>.<minor>.<patch>`)
2. Verifies `pyproject.toml` version matches the tag
3. Generates a changelog from conventional commits (git-cliff)
4. Builds wheels for Linux x86\_64, macOS Apple Silicon, macOS Intel
5. Builds an sdist
6. Creates a GitHub Release with all artifacts attached

### 4. Verify the release

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

For urgent fixes on an already-released version:

```bash
git checkout main
git pull
# make your fix, open a PR, merge it
make bump-patch
```

There is no separate hotfix branch — all releases go through main.

---

## If the release workflow fails

The most common causes:

- **Tag/version mismatch** — the tag was created without running `make bump-*`. Delete the tag, run the bump command, and re-push.
- **Build failure** — a Rust compilation error in the wheel build. Fix the code, delete the tag, re-tag.

To delete a tag and re-release:

```bash
git tag -d v0.1.1
git push origin :refs/tags/v0.1.1
# fix the issue, then
make bump-patch
```
