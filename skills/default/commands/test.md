Run the project's test suite.

Use the test command appropriate for this project. Check the README or Makefile for
the canonical test invocation.

Common patterns:
```bash
# Rust
cargo test

# Python
uv run pytest
# or: python -m pytest

# Node
npm test

# Make
make test
```

If tests fail:
1. Read the failure output — look for the first failing assertion.
2. Check whether a recent code change broke an assumption the test was making.
3. Run a single test in isolation to confirm the failure reproduces.
4. Fix the root cause; don't suppress the test.
