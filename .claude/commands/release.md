Manage versioning and releases for dex.

## Check current version

```bash
make version
```

## Bump and release (local, must be on clean main)

```bash
make bump-patch   # 0.1.0 → 0.1.1  (fix:/chore:/refactor: commits)
make bump-minor   # 0.1.0 → 0.2.0  (feat: commits)
make bump-major   # 0.1.0 → 1.0.0  (BREAKING CHANGE)
```

Each target: updates version in `pyproject.toml` + `crates/*/Cargo.toml`, commits, tags `vX.Y.Z`, and pushes. The pushed tag triggers the `release.yml` workflow, which builds platform wheels and creates a GitHub Release automatically.

## Automatic release on merge to main

Every merge to `main` triggers `version.yml`, which:
1. Determines the next version from conventional commit messages
2. Bumps all version files and commits `"chore: bump version to X.Y.Z"`
3. Pushes the tag → fires `release.yml` → GitHub Release is created

Commit message → bump type:
- `feat:` → minor
- `fix:`, `chore:`, `refactor:`, `docs:`, `test:` → patch
- `BREAKING CHANGE` in commit footer → major

## Manual version set (e.g. to align to a specific version)

```bash
python3 scripts/bump-version.py 1.0.0
```

Then commit, tag, and push manually, or let CI handle it on next merge.

## Troubleshooting

- `make bump-*` fails with "working tree is dirty" — commit or stash changes first.
- `make bump-*` fails with "not on main branch" — switch to main before releasing.
- Release workflow fails at wheel build — check `PyO3/maturin-action` logs; most failures are Rust compile errors visible in the `cargo build` step.
- Tag already exists — delete it locally and remotely: `git tag -d vX.Y.Z && git push origin :refs/tags/vX.Y.Z`
