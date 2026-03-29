#!/usr/bin/env bash
# Update version across Cargo.toml files.
#
# Usage:
#   scripts/bump-version.sh              # print current version
#   scripts/bump-version.sh patch        # 0.1.0 -> 0.1.1
#   scripts/bump-version.sh minor        # 0.1.0 -> 0.2.0
#   scripts/bump-version.sh major        # 0.1.0 -> 1.0.0
#   scripts/bump-version.sh 0.2.0        # set exact version
#   scripts/bump-version.sh v0.2.0       # v-prefix is stripped

set -euo pipefail

CARGO_FILES=(
    "crates/dex-core/Cargo.toml"
    "crates/dex-cli/Cargo.toml"
)

current_version() {
    grep -m1 '^version = ' "${CARGO_FILES[0]}" | sed 's/version = "\(.*\)"/\1/'
}

bump() {
    local current="$1" kind="$2"
    local major minor patch
    IFS='.' read -r major minor patch <<< "$current"
    case "$kind" in
        major) echo "$((major + 1)).0.0" ;;
        minor) echo "${major}.$((minor + 1)).0" ;;
        patch) echo "${major}.${minor}.$((patch + 1))" ;;
    esac
}

set_version() {
    local new="$1"
    for f in "${CARGO_FILES[@]}"; do
        awk -v ver="$new" 'BEGIN{done=0} /^version = / && !done {sub(/"[^"]*"/, "\"" ver "\""); done=1} {print}' \
            "$f" > "$f.tmp" && mv "$f.tmp" "$f"
    done
}

if [ $# -eq 0 ]; then
    current_version
    exit 0
fi

arg="${1#v}"  # strip leading v

case "$arg" in
    patch|minor|major)
        new=$(bump "$(current_version)" "$arg")
        ;;
    [0-9]*\.[0-9]*\.[0-9]*)
        new="$arg"
        ;;
    *)
        echo "error: invalid argument '$arg' — expected X.Y.Z or patch/minor/major" >&2
        exit 1
        ;;
esac

set_version "$new"
echo "$new"
