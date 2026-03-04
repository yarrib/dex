# Installation

dex is distributed as pre-built wheels on [GitHub Releases](https://github.com/yarrib/dex/releases).
No PyPI, no crates.io, no Rust toolchain required.

## One-line install (recommended)

Paste this in your terminal on macOS or Linux:

```bash
curl -sSf https://raw.githubusercontent.com/yarrib/dex/main/install.sh | sh
```

The script will:

1. Detect your OS and CPU architecture
2. Fetch the latest release from GitHub
3. Install [uv](https://docs.astral.sh/uv/) if it is not already present
4. Run `uv tool install` with the correct wheel

After install, `dex` is available globally:

```bash
dex --help
```

!!! note "Windows"
    The install script does not support Windows. Use the manual install path below.

## Manual install with uv

If you prefer not to pipe into `sh`, or you are on Windows:

1. Go to the [latest release](https://github.com/yarrib/dex/releases/latest)
2. Copy the wheel URL for your platform

| Platform | Filename pattern |
|---|---|
| Linux x86\_64 | `*-manylinux_2_17_x86_64.whl` |
| macOS Apple Silicon | `*-macosx_11_0_arm64.whl` |
| macOS Intel | `*-macosx_10_12_x86_64.whl` |
| Windows x86\_64 | `*-win_amd64.whl` |

3. Install via uv:

```bash
uv tool install "dex @ https://github.com/yarrib/dex/releases/download/vX.Y.Z/<wheel-filename>"
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

## PyPI

Coming soon. Track progress in [GitHub Issues](https://github.com/yarrib/dex/issues).
