#!/usr/bin/env bash
# Verify the workspace version is valid for the tag-driven release flow:
#   - dex-core and dex-cli versions agree
#   - the version is not behind the latest released tag (a version behind the
#     latest tag would make tag-on-merge.yml a silent no-op on merge)
#
# Run in CI (.github/workflows/ci.yml) and locally via `make check-version`.
# Requires full tag history: in CI use actions/checkout with fetch-depth: 0.

set -euo pipefail

ver() { grep -m1 '^version = ' "$1" | sed 's/version = "\(.*\)"/\1/'; }

cli_ver="$(ver crates/dex-cli/Cargo.toml)"
core_ver="$(ver crates/dex-core/Cargo.toml)"

if [ "$cli_ver" != "$core_ver" ]; then
    echo "error: version mismatch — dex-cli is $cli_ver but dex-core is $core_ver" >&2
    echo "       Bump both together: bash scripts/bump-version.sh <patch|minor|major>" >&2
    exit 1
fi

# Highest released tag, v-stripped. Empty when the repo has no tags yet.
latest_tag="$(git tag -l 'v*' | sed 's/^v//' | sort -V | tail -1)"

if [ -z "$latest_tag" ]; then
    echo "ok: version $cli_ver (no existing tags)"
    exit 0
fi

# Fail if the current version is strictly less than the latest tag.
highest="$(printf '%s\n%s\n' "$cli_ver" "$latest_tag" | sort -V | tail -1)"
if [ "$cli_ver" != "$latest_tag" ] && [ "$highest" = "$latest_tag" ]; then
    echo "error: version $cli_ver is behind the latest tag v$latest_tag." >&2
    echo "       A release can't go backwards. Bump past v$latest_tag:" >&2
    echo "       make bump-patch | bump-minor | bump-major" >&2
    exit 1
fi

if [ "$cli_ver" = "$latest_tag" ]; then
    echo "ok: version $cli_ver matches latest tag v$latest_tag — merging won't release"
else
    echo "ok: version $cli_ver is ahead of latest tag v$latest_tag — merging will release v$cli_ver"
fi
