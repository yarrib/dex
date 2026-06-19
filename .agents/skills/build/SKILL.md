---
name: build
description: Build dex — the Rust workspace and release binary
---

Build dex. It is 100% Rust — a single binary, no Python runtime required.

```bash
cargo build              # debug build of the whole workspace
cargo build --release    # release binary at target/release/dex
```

To install the binary onto your PATH:

```bash
cargo install --path crates/dex-cli
```

If the build fails:
1. Read the first `cargo build` error — it is usually the root cause; later errors often cascade from it.
2. Confirm your toolchain is current (`rustup update`); dex targets stable Rust, Edition 2024.
3. If a dependency fails to resolve, check `Cargo.lock` is committed and run `cargo fetch`.
