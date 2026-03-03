# Contributing

## Dev setup

Requires [Rust](https://rustup.rs/) and [uv](https://docs.astral.sh/uv/).

```bash
git clone https://github.com/yarrib/dex
cd dex
make dev
```

## Make targets

| Target | Description |
|---|---|
| `make dev` | `uv sync` + `maturin develop` |
| `make build` | `cargo build` + `maturin develop` |
| `make test` | `cargo test` + `uv run pytest` |
| `make lint` | `cargo clippy -D warnings` + `uv run ruff check python/` |
| `make fmt` | `cargo fmt` + `uv run ruff format python/` |
| `make fmt-check` | Format check only (no writes) |
| `make clean` | Remove build artifacts |

## Architecture

```
crates/dex-core/    Rust library — all business logic, no UI
crates/dex-py/      PyO3 bindings — thin FFI layer, type conversion only
python/dex/         Python package — click CLI, rich output, pass-throughs
templates/          Built-in Jinja2 templates, embedded at compile time
```

**Rules:**

- `dex-core` has no terminal output. It returns data; Python renders it.
- `dex-py` is a thin bridge. Business logic belongs in `dex-core`.
- Config is TOML. No YAML, no JSON for config.
- Template files use `.j2` extension (Jinja2/minijinja syntax).
- No `unwrap()` or `expect()` in library code — propagate with `?`.

## Adding a template

1. Create `templates/<name>/template.toml` (see [Template Reference](usage/templates.md))
2. Create `templates/<name>/files/` with Jinja2 template files
3. Run `make build` to embed the template in the binary
4. Test with `dex init --template <name>`

## Commit conventions

```
feat:      new feature
fix:       bug fix
refactor:  code change without behaviour change
docs:      documentation only
test:      tests only
chore:     build, deps, tooling
```
