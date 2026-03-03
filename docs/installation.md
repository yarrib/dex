# Installation

dex ships as pre-built wheels on [GitHub Releases](https://github.com/yarrib/dex/releases).
No Rust toolchain required.

## pip

Download the wheel for your platform from the [latest release](https://github.com/yarrib/dex/releases/latest),
then install it directly:

```bash
pip install https://github.com/yarrib/dex/releases/download/v0.1.0/dex-0.1.0-cp312-cp312-manylinux_2_17_x86_64.whl
```

Replace the filename with the one matching your platform:

| Platform | Filename pattern |
|---|---|
| Linux x86\_64 | `*-manylinux_2_17_x86_64.whl` |
| macOS Apple Silicon | `*-macosx_11_0_arm64.whl` |
| macOS Intel | `*-macosx_10_12_x86_64.whl` |
| Windows x86\_64 | `*-win_amd64.whl` |

## uv

```bash
uv tool install "dex @ https://github.com/yarrib/dex/releases/download/v0.1.0/dex-0.1.0-cp312-cp312-manylinux_2_17_x86_64.whl"
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

After that, the `dex` command is available:

```bash
dex --help
```

## Requirements

- Python 3.11+
- For building from source: Rust stable (`rustup update stable`)
