# CLAUDE.md — Development Rules for dex

## What is dex

An extensible CLI framework for data project operations — Python packages, Databricks workflows,
and more. Rust core (template engine, config, file I/O) with Python surface (click CLI,
extensibility, pass-throughs). Distributed as a Python package via maturin/PyO3.
See `docs/SPEC.md` and `docs/ARCHITECTURE.md`.

## Build Commands

```bash
# Rust
cargo build                      # build all crates
cargo test                       # run all Rust tests
cargo clippy -- -D warnings      # lint (treat warnings as errors)
cargo fmt --check                # format check

# Python (requires maturin + uv)
uv sync                          # install Python deps
maturin develop                  # build + install Rust extension into venv
uv run pytest                    # run Python tests
uv run ruff check python/        # lint Python
uv run ruff format --check python/  # format check Python

# Shortcuts via make
make dev                         # uv sync + maturin develop
make build                       # cargo build + maturin develop
make test                        # cargo test + uv run pytest
make lint                        # clippy + ruff check
make fmt                         # cargo fmt + ruff format
make fmt-check                   # format check (no writes)
```

## Repository Structure

```
crates/dex-core/    Rust library. All business logic. No UI, no Python deps.
crates/dex-py/      PyO3 bindings. Thin FFI layer. Delegates to dex-core.
python/dex/         Python package. CLI (click), extensions, pass-throughs.
templates/          Built-in templates. Embedded at compile time via include_dir.
docs/               Specification and architecture documents.
```

## Architectural Rules

1. **dex-core has no UI.** No terminal colors, no prompts, no spinners. It returns data;
   the Python layer renders it. This keeps the core testable and the FFI boundary clean.

2. **dex-py is a thin bridge.** It converts types and delegates. No business logic.
   If you're writing more than type conversion in dex-py, it belongs in dex-core.

3. **Python owns all user interaction.** Prompts, output formatting, progress indicators,
   error display — all in `python/dex/`. The Rust core never talks to the terminal.

4. **Pass-throughs are Python-only.** They're subprocess calls. No Rust involvement.

5. **Config is TOML.** Project config is `dex.toml`. Template manifests are `template.toml`.
   User config is `~/.config/dex/config.toml`. No YAML, no JSON for config.

6. **Templates use Jinja2 syntax.** Rendered by minijinja in Rust. File extension `.j2`
   for template files. This keeps the syntax familiar to Python users.

7. **Errors cross the FFI boundary as strings.** Rust errors (thiserror) are converted
   to Python exceptions in dex-py. Keep error messages user-facing and actionable.

## Coding Conventions

### Rust

- Edition 2024, target stable Rust
- `thiserror` for error types in dex-core, never `anyhow`
- `#[must_use]` on functions that return values callers shouldn't ignore
- Public API types in `lib.rs`, implementation in submodules
- Tests in the same file (`#[cfg(test)] mod tests`), integration tests in `tests/`
- No `unwrap()` or `expect()` in library code — propagate errors with `?`
- Prefer `&str` over `String` in function parameters where ownership isn't needed

### Python

- Python 3.11+
- Type hints on all public functions
- `click` for CLI, not `argparse` or `typer`
- `rich` for terminal output formatting
- No classes where a function will do
- Test with `pytest` and `click.testing.CliRunner`

## Commit Conventions

- Prefix: `feat:`, `fix:`, `refactor:`, `docs:`, `test:`, `chore:`
- Scope is optional: `feat(core):`, `fix(cli):`
- Imperative mood: "add template rendering" not "added template rendering"
- One logical change per commit

## Adding a New Subcommand

1. Add core logic to `crates/dex-core/src/` (new module or extend existing)
2. Expose via `dex-core`'s public API in `lib.rs`
3. Add PyO3 binding in `crates/dex-py/src/lib.rs`
4. Add click command in `python/dex/cli.py`
5. Add tests at each layer
6. Update `docs/SPEC.md` with the command's interface
