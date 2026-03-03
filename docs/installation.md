# Installation

## pip

```bash
pip install dex
```

## uv

```bash
uv add dex
```

## Build from source

Requires [Rust](https://rustup.rs/) and [uv](https://docs.astral.sh/uv/).

```bash
git clone https://github.com/yarrib/dex
cd dex
make dev
```

`make dev` runs `uv sync` followed by `maturin develop`, which compiles the Rust extension and installs everything into the local virtual environment.

After that, the `dex` command is available:

```bash
dex --help
```

## Requirements

- Python 3.11+
- For building from source: Rust stable (`rustup update stable`)
