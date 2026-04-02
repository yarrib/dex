Build the project.

Run the build command appropriate for this project. Check the README or Makefile for the
canonical build invocation.

Common patterns:
```bash
# Rust
cargo build

# Python (with uv)
uv sync

# Node
npm install && npm run build

# Make
make build
```

If the build fails:
1. Read the error output carefully — the first error is usually the root cause.
2. Check that all dependencies are installed.
3. For compiled languages, ensure the toolchain version matches what the project expects.
