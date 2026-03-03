#!/usr/bin/env python3
"""Update version across pyproject.toml and Cargo.toml files.

Usage:
    python3 scripts/bump-version.py              # print current version
    python3 scripts/bump-version.py patch        # 0.1.0 -> 0.1.1
    python3 scripts/bump-version.py minor        # 0.1.0 -> 0.2.0
    python3 scripts/bump-version.py major        # 0.1.0 -> 1.0.0
    python3 scripts/bump-version.py 0.2.0        # set exact version
    python3 scripts/bump-version.py v0.2.0       # v-prefix is stripped
"""

import re
import sys
from pathlib import Path

PYPROJECT = Path("pyproject.toml")
CARGO_FILES = [
    Path("crates/dex-core/Cargo.toml"),
    Path("crates/dex-py/Cargo.toml"),
]


def current_version() -> str:
    text = PYPROJECT.read_text()
    m = re.search(r'^version = "(.+?)"', text, re.MULTILINE)
    if not m:
        sys.exit("error: version not found in pyproject.toml")
    return m.group(1)


def next_version(current: str, bump: str) -> str:
    major, minor, patch = map(int, current.split("."))
    if bump == "major":
        return f"{major + 1}.0.0"
    if bump == "minor":
        return f"{major}.{minor + 1}.0"
    return f"{major}.{minor}.{patch + 1}"


def set_version(path: Path, new: str) -> None:
    text = path.read_text()
    updated = re.sub(
        r'^version = "\d+\.\d+\.\d+"',
        f'version = "{new}"',
        text,
        count=1,
        flags=re.MULTILINE,
    )
    if updated == text:
        sys.exit(f"error: could not find version field in {path}")
    path.write_text(updated)


def main() -> None:
    if len(sys.argv) < 2:
        print(current_version())
        return

    arg = sys.argv[1].lstrip("v")

    if arg in ("patch", "minor", "major"):
        new = next_version(current_version(), arg)
    else:
        if not re.fullmatch(r"\d+\.\d+\.\d+", arg):
            sys.exit(f"error: invalid version '{arg}' — expected X.Y.Z or patch/minor/major")
        new = arg

    set_version(PYPROJECT, new)
    for cargo in CARGO_FILES:
        set_version(cargo, new)

    print(new)


if __name__ == "__main__":
    main()
