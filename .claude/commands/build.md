Run the full build: Rust and Python extension.

```bash
make build
```

This runs `cargo build` followed by `maturin develop` to compile the Rust extension
and install it into the current virtual environment.

If the build fails:
1. Check `cargo build` errors first — Rust errors are the most common root cause.
2. If `maturin develop` fails, ensure you have an active Python 3.12+ virtual environment.
3. Run `uv sync` to ensure Python deps are installed before `maturin develop`.
