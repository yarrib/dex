# Installation

dex is distributed as pre-built wheels on [GitHub Releases](https://github.com/yarrib/dex/releases).
No PyPI, no crates.io, no Rust toolchain required.

## Install with uv (recommended)

`uv tool install` installs dex as an isolated CLI tool — the same way you'd install `ruff` or `mypy`.

Go to the [latest release](https://github.com/yarrib/dex/releases/latest) and copy the wheel URL
for your platform, then run:

```bash
uv tool install "dex @ https://github.com/yarrib/dex/releases/download/vX.Y.Z/<wheel-filename>"
```

Replace `<wheel-filename>` with the one matching your platform:

| Platform | Filename pattern |
|---|---|
| Linux x86\_64 | `*-manylinux_2_17_x86_64.whl` |
| macOS Apple Silicon | `*-macosx_11_0_arm64.whl` |
| macOS Intel | `*-macosx_10_12_x86_64.whl` |
| Windows x86\_64 | `*-win_amd64.whl` |

After install, `dex` is available globally:

```bash
dex --help
```

## Upgrade

```bash
uv tool upgrade dex
```

Or reinstall from a specific release:

```bash
uv tool install --force "dex @ https://github.com/yarrib/dex/releases/download/vX.Y.Z/<wheel-filename>"
```

## Uninstall

```bash
uv tool uninstall dex
```

## Build from source

Requires [Rust](https://rustup.rs/) and [uv](https://docs.astral.sh/uv/).

```bash
git clone https://github.com/yarrib/dex
cd dex
make dev
```

`make dev` runs `uv sync` followed by `maturin develop`, which compiles the Rust extension
and installs everything into the local virtual environment.

After that, `dex` is available in the venv:

```bash
dex --help
```

## Requirements

- Python 3.11+
- [uv](https://docs.astral.sh/uv/) — for `uv tool install`
- For building from source: Rust stable (`rustup update stable`)
