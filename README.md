# dex — data extensions

Extensible CLI framework for data project operations.

> Scaffold Python packages, Databricks workflows, and more — then extend it for your org.

Rust core for performance. Python surface for extensibility. Zero-compromise ergonomics.

## Install

```bash
uv tool install dex
```

## Quick Start

```bash
# Scaffold a new project
dex init --template default

# Or build your own org CLI on top
uv add dex
```

```python
from dex.cli import create_cli
from dex.passthrough import PassthroughSpec

cli = create_cli(
    name="acme-dex",
    passthroughs=[
        PassthroughSpec(name="db", command="databricks", description="Databricks CLI"),
    ],
)

@cli.command()
def deploy():
    """Custom deploy logic for your org."""
    ...
```

## Development

Requires Rust stable and [uv](https://github.com/astral-sh/uv).

```bash
git clone https://github.com/yarrib/dex
cd dex
make dev          # uv sync + maturin develop (builds Rust extension into venv)
make test         # cargo test + uv run pytest
make lint         # cargo clippy + ruff check
```

Common targets:

| Target | What it does |
|--------|-------------|
| `make dev` | Install deps and build Rust extension |
| `make build` | `cargo build` + `maturin develop` |
| `make test` | Full test suite (Rust + Python) |
| `make lint` | `clippy -D warnings` + `ruff check` |
| `make fmt` | Auto-format Rust + Python |
| `make fmt-check` | Format check without writing |
| `make clean` | Remove build artefacts |

See [docs/SPEC.md](docs/SPEC.md) and [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for
full specification and architecture.

## Status

**v0.1 — in development.** `dex init` with template scaffolding (Python packages, Databricks Asset Bundles).

## License

MIT
