#!/usr/bin/env bash
# Open a release PR: branch off main, bump the version, commit, push, open a PR.
#
# Usage:
#   scripts/release.sh patch        # 0.2.0 -> 0.2.1  (fix:/chore:/refactor:/docs:/test:)
#   scripts/release.sh minor        # 0.2.0 -> 0.3.0  (feat:)
#   scripts/release.sh major        # 0.2.0 -> 1.0.0  (BREAKING CHANGE)
#
# After the PR merges to main, .github/workflows/tag-on-merge.yml tags the
# release and dispatches release.yml automatically. No manual tagging needed.

set -euo pipefail

kind="${1:-}"
case "$kind" in
    patch|minor|major) ;;
    *) echo "usage: scripts/release.sh patch|minor|major" >&2; exit 1 ;;
esac

# Start from a clean, up-to-date main so the PR base is correct.
git diff --quiet && git diff --staged --quiet \
    || { echo "error: working tree is dirty — commit or stash first"; exit 1; }

current_branch="$(git branch --show-current)"
if [ "$current_branch" != "main" ]; then
    echo "error: run from main (you are on '$current_branch'). Try: git checkout main && git pull" >&2
    exit 1
fi
git pull --ff-only origin main

new="$(bash scripts/bump-version.sh "$kind")"
branch="chore/release-v${new}"

git checkout -b "$branch"
git add crates/dex-core/Cargo.toml crates/dex-cli/Cargo.toml Cargo.lock
git commit -m "chore: bump version to v${new}"
git push -u origin "$branch"

if command -v gh >/dev/null 2>&1; then
    gh pr create --base main --head "$branch" \
        --title "chore: release v${new}" \
        --body "Bumps the version to v${new}.

Merging this PR triggers \`tag-on-merge.yml\`, which tags \`v${new}\` and dispatches the release workflow."
    echo "Opened release PR for v${new}. Merge it to ship the release."
else
    echo "Pushed ${branch}. gh CLI not found — open a PR into main manually."
    echo "After it merges, tag-on-merge.yml tags v${new} and runs the release."
fi
