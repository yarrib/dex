Run all linters: Rust clippy and Python ruff.

```bash
make lint
```

This runs `cargo clippy -- -D warnings` (warnings are errors) and `uv run ruff check python/`.

To auto-fix Python lint issues:
```bash
uv run ruff check python/ --fix
```

To auto-format:
```bash
make fmt
```

To check formatting without writing:
```bash
make fmt-check
```

Common issues:
- `clippy` flag `#[must_use]` — add the attribute to functions returning values callers shouldn't ignore.
- `unwrap()` or `expect()` in library code — propagate with `?` instead.
- `ruff` import sorting — run `uv run ruff check python/ --fix` to resolve.
