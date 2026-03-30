# Contributing

## Dev setup

Requires [Rust](https://rustup.rs/) (stable).

```bash
git clone https://github.com/yarrib/dex
cd dex
cargo build
```

## Make targets

| Target | Description |
|---|---|
| `make build` | `cargo build` |
| `make test` | `cargo test` |
| `make lint` | `cargo clippy -- -D warnings` |
| `make fmt` | `cargo fmt` |
| `make fmt-check` | Format check only (no writes) |
| `make clean` | Remove build artifacts |
| `make docs` | Build docs site |
| `make docs-serve` | Serve docs site at localhost:3000 |

## Architecture

```
crates/dex-core/    Rust library — all business logic, no UI
crates/dex-cli/     Rust binary — clap CLI, dialoguer prompts, console output
templates/          Built-in Jinja2 templates, embedded at compile time
```

**Rules:**

- `dex-core` has no terminal output. It returns data; `dex-cli` renders it.
- `dex-cli` owns all user interaction: prompts, formatting, error display.
- Config is TOML. No YAML, no JSON for config.
- Template files use `.j2` extension (Jinja2/minijinja syntax).
- No `unwrap()` or `expect()` in library code — propagate with `?`.

## Adding a template

1. Create `templates/<name>/template.toml` (see [Template Reference](usage/templates.md))
2. Create `templates/<name>/files/` with Jinja2 template files
3. Run `make build` to embed the template in the binary
4. Test with `dex init --template <name>`

## Adding a subcommand

1. Add core logic to `crates/dex-core/src/` (new module or extend existing)
2. Expose via `dex-core`'s public API in `lib.rs`
3. Add clap command in `crates/dex-cli/src/commands/`
4. Register in `crates/dex-cli/src/main.rs`
5. Add tests at each layer
6. Update `docs/SPEC.md` with the command's interface

## Docs

The docs site uses [mdBook](https://rust-lang.github.io/mdBook/) and is served via GitHub Pages.

**Local preview:**

```bash
make docs-install   # install mdbook (once)
make docs-serve     # browse docs at localhost:3000
```

## Commit conventions

```
feat:      new feature
fix:       bug fix
refactor:  code change without behaviour change
docs:      documentation only
test:      tests only
chore:     build, deps, tooling
```
