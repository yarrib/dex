Run linters and formatters.

Use the lint/format commands appropriate for this project.

Common patterns:
```bash
# Rust
cargo clippy -- -D warnings
cargo fmt --check

# Python
uv run ruff check .
uv run ruff format --check .

# Node / TypeScript
npm run lint

# Make
make lint
```

To auto-fix issues:
```bash
# Rust
cargo fmt

# Python
uv run ruff check . --fix
uv run ruff format .
```

Common issues:
- Unused imports or variables — remove them.
- `unwrap()` / `expect()` in library code — propagate errors with `?` instead.
- Import ordering — run the formatter to fix automatically.
