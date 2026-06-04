Manage versioning and releases for dex.

Releases are **tag-driven**: pushing a `vX.Y.Z` tag to `main` triggers the
`release.yml` workflow, which validates the tag, generates a changelog, builds
the native binaries (no wheels — dex ships a single Rust binary), and publishes
a GitHub Release. Because `main` is protected, the version bump goes through a PR
first; the tag is pushed only after that PR merges.

## 1. Check the current version

```bash
make version            # or: bash scripts/bump-version.sh
```

## 2. Bump the version on a release branch

```bash
git checkout main && git pull
git checkout -b chore/release-vX.Y.Z

make bump-patch         # 0.2.0 → 0.2.1  (fix:/chore:/refactor:/docs:/test:)
make bump-minor         # 0.2.0 → 0.3.0  (feat:)
make bump-major         # 0.2.0 → 1.0.0  (BREAKING CHANGE)

git push -u origin chore/release-vX.Y.Z
```

Each `bump-*` target runs `scripts/bump-version.sh`, which updates the version in
`crates/dex-core/Cargo.toml`, `crates/dex-cli/Cargo.toml`, and `Cargo.lock`, then
commits `chore: bump version to vX.Y.Z`. The guard aborts if the working tree is
dirty or you're not on a release branch.

Open a PR for the branch and get it merged into `main`.

## 3. Tag main and trigger the release

After the bump PR is merged:

```bash
git checkout main && git pull
make tag-release        # must be on main; tags HEAD with the Cargo.toml version and pushes
```

`make tag-release` annotates the tag with a `git-cliff` changelog when git-cliff
is installed locally; otherwise it pushes a lightweight tag and the workflow
promotes it to an annotated tag with the same changelog. The pushed tag fires
`release.yml`.

Alternatively, trigger the workflow manually from **Actions → Release →
Run workflow** (`workflow_dispatch`) with the version (e.g. `0.2.1`). It still
validates that the version matches `crates/dex-cli/Cargo.toml` on `main`, so the
bump PR must be merged first either way.

## 4. What `release.yml` does

1. Resolves the tag/version (from the pushed tag or the `workflow_dispatch` input)
2. Validates the tag format `v<major>.<minor>.<patch>` and that it matches
   `crates/dex-cli/Cargo.toml`
3. Generates a changelog from conventional commits with `git-cliff`
4. Builds native binaries: `dex-linux-x86_64`, `dex-linux-aarch64` (musl, via
   `cross`), `dex-macos-x86_64`, `dex-macos-aarch64`
5. Creates the GitHub Release with the binaries attached and the changelog body

## Troubleshooting

- **`bump-*` fails "working tree is dirty"** — commit or stash changes first.
- **`tag-release` fails "must be on main branch"** — switch to `main` after the
  bump PR merges.
- **Release workflow fails at "Verify Cargo.toml version matches tag"** — the tag
  was pushed before the bump merged to `main`. Bump + merge first, then re-tag.
- **Tag already exists** — delete it locally and remotely, then re-tag:
  `git tag -d vX.Y.Z && git push origin :refs/tags/vX.Y.Z`
