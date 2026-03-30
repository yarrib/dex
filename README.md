# dex — data extensions

Extensible CLI framework for data project operations.

> Scaffold Python packages, Databricks workflows, and more — then extend it for your org.

100% Rust. Single binary. No runtime required.

## Install

```bash
curl -sSf https://raw.githubusercontent.com/yarrib/dex/main/install.sh | sh
```

Auto-detects your platform and downloads the right binary from GitHub Releases.

## Quick Start

```bash
# Scaffold a new project
dex init --template dabs-package --dir my_project

# Add pass-throughs and custom templates via dex.toml
cat dex.toml
```

```toml
[passthrough.db]
command = "databricks"
description = "Databricks CLI"

[passthrough.tf]
command = "terraform"
description = "Terraform"
```

```bash
dex db clusters list    # → databricks clusters list
dex tf plan             # → terraform plan
```

See [Extending dex](docs/extending.md) and [Building Org Templates](docs/templates/org-templates-guide.md) for how to share templates and pass-throughs across your team.

## Development

Requires [Rust](https://rustup.rs/) stable.

```bash
git clone https://github.com/yarrib/dex
cd dex
cargo build
cargo test
```

Common targets:

| Target | What it does |
|--------|-------------|
| `make build` | `cargo build` |
| `make test` | `cargo test` |
| `make lint` | `cargo clippy -- -D warnings` |
| `make fmt` | `cargo fmt` |
| `make fmt-check` | Format check without writing |
| `make clean` | Remove build artefacts |
| `make docs-serve` | Serve docs at localhost:3000 |

See [docs/SPEC.md](docs/SPEC.md) and [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for
full specification and architecture.

## Status

**v0.1.0** — `dex init` with template scaffolding (Python packages, Databricks Asset Bundles).

## License

MIT
