Run the full test suite: Rust unit tests and Python integration tests.

```bash
make test
```

This runs `cargo test` (all Rust crates) followed by `uv run pytest` (Python tests in `tests/`).

To run only Rust tests:
```bash
cargo test
```

To run only Python tests:
```bash
uv run pytest
```

To run a specific Python test:
```bash
uv run pytest tests/test_cli.py -v
```

If tests fail:
1. Rebuild first: `make build` (Python tests depend on the compiled `.so`).
2. Check whether a Rust API change broke the PyO3 bindings in `crates/dex-py/src/lib.rs`.
3. Rust tests are in the same file as the implementation (`#[cfg(test)] mod tests`).
