# CLAUDE.md — Development Rules for dex

## What is dex

An extensible CLI framework for data project operations — Python packages, Databricks workflows,
and more. 100% Rust, single binary. Extensibility via templates (minijinja) and pass-through
commands (dex.toml config). Distributed as a standalone binary.

See `docs/SPEC.md` and `docs/ARCHITECTURE.md`.

## Build Commands

```bash
# Build
cargo build                      # build all crates
cargo build --release            # release build (produces target/release/dex)

# Test
cargo test                       # run all Rust tests

# Lint & format
cargo clippy -- -D warnings      # lint (treat warnings as errors)
cargo fmt --check                # format check
```

## Repository Structure

```
crates/dex-core/    Rust library. All business logic. No UI, no terminal output.
crates/dex-cli/     Binary crate. CLI (clap), interactive prompts (dialoguer), terminal output (console).
crates/dex-py/      PyO3 bindings. Optional thin FFI layer for Python interop (not required).
templates/          Built-in templates. Embedded at compile time via include_dir.
docs/               Specification and architecture documents.
```

## Architectural Rules

1. **dex-core has no UI.** No terminal colors, no prompts, no spinners. It returns data;
   the CLI layer renders it. This keeps the core testable and reusable.

2. **dex-cli owns all user interaction.** Prompts (dialoguer), output formatting (console),
   progress indicators (indicatif), error display — all in `crates/dex-cli/`.

3. **dex-py is optional.** It exists for backwards compatibility but is not required.
   The primary distribution is the native `dex` binary from dex-cli.

4. **Pass-throughs are config-driven.** Defined in `[passthrough]` in `dex.toml`.
   They delegate to external CLIs via `std::process::Command`.

5. **Extensibility via templates and config.** Orgs customize dex through:
   - Custom templates (directory or remote git repos)
   - Pass-through commands in `dex.toml`
   - Standards files for variable pre-fills
   No plugin system or code-based extension needed.

6. **Config is TOML.** Project config is `dex.toml`. Template manifests are `template.toml`.
   User config is `~/.config/dex/config.toml`. No YAML, no JSON for config.

7. **Templates use Jinja2 syntax.** Rendered by minijinja in Rust. File extension `.j2`
   for template files. Familiar to Python users.

8. **Errors are propagated, not panicked.** Use `thiserror` in dex-core, `?` for propagation.
   No `unwrap()` or `expect()` in library code.

## Coding Conventions

### Rust

- Edition 2024, target stable Rust
- `thiserror` for error types in dex-core, never `anyhow`
- `#[must_use]` on functions that return values callers shouldn't ignore
- Public API types in `lib.rs`, implementation in submodules
- Tests in the same file (`#[cfg(test)] mod tests`), integration tests in `tests/`
- No `unwrap()` or `expect()` in library code — propagate errors with `?`
- Prefer `&str` over `String` in function parameters where ownership isn't needed
- `clap` with derive macros for CLI argument parsing
- `dialoguer` for interactive prompts
- `console` for terminal styling

## Adding a New Subcommand

1. Add core logic to `crates/dex-core/src/` (new module or extend existing)
2. Expose via `dex-core`'s public API in `lib.rs`
3. Add clap command in `crates/dex-cli/src/commands/`
4. Register in `crates/dex-cli/src/main.rs`
5. Add tests at each layer
6. Update `docs/SPEC.md` with the command's interface

## Release Process

Releases are tag-driven. Because main is protected, version bumps go through a PR:

```bash
# 1. On a release branch, bump versions in Cargo.toml files:
git checkout -b chore/release-v0.x.y
# Update version in workspace and crate Cargo.toml files
git commit -m "chore: bump version to 0.x.y"
git push -u origin chore/release-v0.x.y

# 2. After merging, tag main:
git checkout main && git pull
git tag v0.x.y && git push origin v0.x.y
```

## Git Workflow

- **main is protected.** Never push directly to main.
- **All work goes on branches.** Branch from main: `feat/`, `fix/`, `chore/`, `docs/`.
- **PRs are required.** All CI checks must pass before merging.
- **One logical change per PR.** Keep PRs small and focused.

## Commit Conventions

- Prefix: `feat:`, `fix:`, `refactor:`, `docs:`, `test:`, `chore:`
- Scope is optional: `feat(core):`, `fix(cli):`
- Imperative mood: "add template rendering" not "added template rendering"
- One logical change per commit

## Distribution

The primary distribution is a single native binary. No Python runtime required.

```bash
# Install from source
cargo install --path crates/dex-cli

# Or build release binary
cargo build --release
# Binary at: target/release/dex
```
