# dex

Opinionated CLI framework for Databricks/MLOps project operations.

> What uv did for Python packaging, dex does for Databricks project operations.

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

## Status

**v0.1 — in development.** `dex init` with template scaffolding.

See [docs/SPEC.md](docs/SPEC.md) and [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for
full specification and architecture.

## License

MIT
