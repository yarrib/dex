# PRD: AI-Ready Scaffolding (2026)

**Status:** Planning
**Owner:** TBD

---

## Overview

To position dex as competitive against AI-native editors (Cursor, GitHub Copilot Workspace,
Windsurf), this document captures three capability areas that make dex scaffolding
immediately usable by AI agents and web-based environments.

---

## 1. Context-Map Generation

### Problem

AI agents (Claude Code, Codex, Gemini) spend tokens and round-trips scanning scaffolded
projects before they can act. There is no machine-readable index of what was created,
what each file does, or how pieces relate.

### Proposal

After every `dex init` (and `dex add`, see §2), emit a `.context-map.json` at the
project root that describes the scaffold output in a format optimized for LLM consumption.

### Schema

```json
{
  "schema_version": "1",
  "generated_by": "dex 0.4.0",
  "template": "dabs-etl",
  "scaffolded_at": "2026-04-02T00:00:00Z",
  "variables": {
    "project_name": "user_events_pipeline",
    "python_version": "3.12"
  },
  "files": [
    {
      "path": "src/user_events_pipeline/main.py",
      "role": "entry_point",
      "description": "Main pipeline logic. Edit this to implement your ETL."
    },
    {
      "path": "dex.toml",
      "role": "config",
      "description": "dex project config. Defines tasks and pass-through commands."
    },
    {
      "path": "databricks.yml",
      "role": "bundle_config",
      "description": "Databricks Asset Bundle definition."
    }
  ],
  "entry_points": ["src/user_events_pipeline/main.py"],
  "tasks": ["test", "lint", "deploy"],
  "traits": []
}
```

### Template Integration

Templates opt in via `template.toml`:

```toml
[[files]]
src = "src/{{ project_name }}/main.py.j2"
context_role = "entry_point"
context_description = "Main pipeline logic. Edit this to implement your ETL."
```

Files without `context_role` are still listed in `.files[]` with `role = "other"`.

### Implementation Plan

1. **dex-core**: Add `ContextMap` struct and serialization to `dex_core::context`.
2. **dex-core**: Extend `ScaffoldOutput` (returned by `scaffold()`) to carry per-file
   metadata sourced from `template.toml` `[[files]]` annotations.
3. **dex-core**: `write_context_map(output: &ScaffoldOutput, dir: &Path)` writes
   `.context-map.json` after scaffold completes.
4. **dex-cli**: Call `write_context_map` in the `init` command handler after scaffold.
5. **MCP**: `scaffold_project` response includes `context_map` field (the parsed JSON).
6. **Templates**: Annotate built-in templates with `context_role` / `context_description`.

### Out of Scope

- Updating `.context-map.json` automatically when files change outside of dex.
- Semantic analysis of file contents (dex only records what it knows at scaffold time).

---

## 2. Modular Traits (`dex add`)

### Problem

Static templates are all-or-nothing. Teams often want to start with a minimal scaffold
and bolt on standardized components (Docker support, auth, CI config) incrementally,
without copy-pasting boilerplate.

### Proposal

`dex add <trait>` injects a named, versioned component into an existing dex project.
Traits are mini-templates: they have their own `trait.toml` manifest, variable declarations,
and file set. They are aware of the existing project and can append to existing files
(e.g., add a service to `databricks.yml`).

### CLI

```
dex add <trait> [--dry-run] [--no-prompt] [--preset <profile>]

Examples:
  dex add docker           # add Dockerfile + .dockerignore
  dex add ci-github        # add .github/workflows/ci.yml
  dex add auth             # add Databricks token auth helpers
  dex add notebook         # add a starter notebook
```

### Trait Manifest: `trait.toml`

```toml
[trait]
name = "docker"
description = "Add Docker support to an existing dex project"
version = "0.1.0"
min_dex_version = "0.4.0"

# Variables follow the same schema as template.toml [[variables]]
[[variables]]
name = "base_image"
prompt = "Base Docker image"
type = "string"
default = "python:{{ python_version }}-slim"

# File injection rules
[[files]]
src = "Dockerfile.j2"
dest = "Dockerfile"
conflict = "error"     # error | overwrite | skip | merge

[[files]]
src = ".dockerignore.j2"
dest = ".dockerignore"
conflict = "skip"

# Patch rules: append content to an existing file
[[patches]]
target = "dex.toml"
append = """
[tasks.docker-build]
command = "docker build -t {{ project_name }} ."
description = "Build Docker image"
"""
```

### Conflict Policy

| `conflict` value | Behavior |
|-----------------|----------|
| `error` (default) | Abort if the destination file exists |
| `overwrite` | Replace the destination file |
| `skip` | Leave existing file unchanged, emit warning |
| `merge` | Reserved for future three-way merge support |

### Trait Sources

Traits are resolved in the same order as templates:
1. Embedded (compiled into binary)
2. Project-local `./traits/` directory
3. User-configured paths in `~/.config/dex/config.toml`

### `dex.toml` Tracking

Applied traits are recorded so `dex add` can detect re-application:

```toml
[project]
name = "my-project"
template = "dabs-etl"
traits = ["docker", "ci-github"]
```

### Implementation Plan

1. **dex-core**: `TraitManifest` struct (mirrors `TemplateManifest`).
2. **dex-core**: `apply_trait(trait: &TraitManifest, dir: &Path, vars: &Vars)` —
   writes files and applies patches. Returns `TraitOutput`.
3. **dex-core**: `PatchRule` — append-to-file logic with TOML-aware appending.
4. **dex-core**: Update `write_context_map` to append applied trait to `traits[]`.
5. **dex-cli**: `dex add` command in `crates/dex-cli/src/commands/add.rs`.
6. **dex-cli**: Reads existing `dex.toml`, checks `traits` list, errors on re-application
   (unless `--force`).
7. **MCP**: Add `apply_trait` tool (v0.4 scope).
8. **Templates**: Ship built-in traits: `docker`, `ci-github`, `notebook`.

### Open Questions

1. Should traits be able to depend on other traits (e.g., `ci-github` requires `docker`)?
   Start with no dependencies; add `requires` field later.
2. TOML-aware patching vs. raw append: raw append is simpler and avoids parse errors.
   Decide at implementation time.
3. Should `--dry-run` print a diff to stdout? Yes — show files that would be created
   and patches that would be applied.

---

## 3. WASM Compatibility

### Problem

Web-based IDEs (GitHub Codespaces, StackBlitz, Replit, CodeSandbox) run in sandboxed
environments where native binaries cannot execute. Users in these environments cannot
install or run `dex` without a container or devcontainer setup.

### Proposal

Compile `dex-core` to WebAssembly (`wasm32-unknown-unknown` or `wasm32-wasi`) and expose
it via a thin JavaScript/TypeScript binding. This enables:

- A `dex` VS Code / Theia extension that works natively in browser-based IDEs.
- A Node.js package (`@dex/core`) for scripting scaffold operations.
- AI agent integrations that run entirely in-browser.

`dex-cli` (which depends on `dialoguer`, `console`, and `std::process`) is **not**
compiled to WASM. Only `dex-core` is targeted.

### Target Environments

| Environment | Target | Binding |
|-------------|--------|---------|
| Node.js / Bun | `wasm32-wasi` | `wasm-bindgen` or raw WASI |
| Browser (VS Code web) | `wasm32-unknown-unknown` | `wasm-bindgen` |
| Deno | `wasm32-wasi` | native WASI import |

### Architecture Constraints

`dex-core` must remain free of:
- `std::process::Command` (process spawning — already absent by architectural rule)
- File system calls that do not abstract over WASI vs. native (`std::fs` must be
  conditionally replaced with a VFS abstraction for the browser target)
- Dependencies that do not compile to `wasm32-unknown-unknown`

### VFS Abstraction

The largest change: `dex-core` currently writes directly to the filesystem via `std::fs`.
For WASM, introduce a `FileSystem` trait:

```rust
pub trait FileSystem {
    fn write_file(&self, path: &Path, content: &[u8]) -> Result<(), FsError>;
    fn read_file(&self, path: &Path) -> Result<Vec<u8>, FsError>;
    fn create_dir_all(&self, path: &Path) -> Result<(), FsError>;
    fn exists(&self, path: &Path) -> bool;
}
```

Implementations:
- `NativeFs` — wraps `std::fs`, used in `dex-cli` and `dex-py`
- `WasiFs` — wraps `wasi::fs`, used in the Node.js/Deno target
- `MemFs` — in-memory, used in unit tests and the browser target

`dex-core` public functions accept `&dyn FileSystem`. This also improves testability.

### JavaScript API (Sketch)

```typescript
import { scaffold, listTemplates } from "@dex/core";

const files = await scaffold({
  template: "dabs-etl",
  directory: "/workspace/my-project",
  variables: { project_name: "my_pipeline", python_version: "3.12" },
});
// files: Array<{ path: string; content: Uint8Array }>
```

### Implementation Plan

1. **dex-core**: Introduce `FileSystem` trait and `NativeFs` implementation.
   Refactor `scaffold()` and `apply_trait()` to accept `&dyn FileSystem`.
2. **dex-core**: Gate `std::fs` usage behind `#[cfg(not(target_arch = "wasm32"))]`.
   Add `MemFs` for tests and browser.
3. **crates/dex-wasm** (new crate): `wasm-bindgen` bindings wrapping `dex-core`.
   Expose `scaffold`, `list_templates`, `apply_trait`, `get_template_variables`.
4. **CI**: Add `wasm32-unknown-unknown` build step. Initially just `cargo build` — no
   test runner needed until a JS test harness is set up.
5. **npm package**: `packages/dex-core-wasm/` — `package.json`, `wasm-bindgen`-generated
   JS glue, TypeScript types.
6. **VS Code extension** (stretch): `extensions/vscode-dex/` — uses `@dex/core` WASM
   package to provide scaffold commands in the command palette.

### Risks

| Risk | Mitigation |
|------|-----------|
| Dependencies that don't compile to WASM | Audit `Cargo.toml` transitive deps early; `minijinja` supports `wasm32` |
| `include_dir` (embedded templates) WASM support | `include_dir` compiles to WASM — templates are embedded at compile time, no FS needed |
| WASI vs. `wasm32-unknown-unknown` split | Ship `wasm32-unknown-unknown` first (broader browser compat); add WASI variant later |
| `wasm-bindgen` API churn | Pin version; isolate in `dex-wasm` crate |

### Out of Scope

- Interactive CLI (`dialoguer`, TTY prompts) in WASM.
- Pass-through commands (`std::process::Command`) in WASM.
- Full `dex-cli` WASM build.

---

## Implementation Sequencing

These three features have a natural dependency order:

1. **Context-Map Generation** first — low risk, high value, no breaking changes.
   Depends only on extending `ScaffoldOutput` and adding a write step.
2. **Modular Traits** second — builds on the same scaffold machinery; context map
   is updated to track traits.
3. **WASM Compatibility** third — requires the `FileSystem` trait refactor, which
   is a larger internal change. Benefits from traits and context map being stable first.

---

## Version Targets

| Feature | Target Version |
|---------|---------------|
| Context-Map Generation | v0.4 |
| Modular Traits (`dex add`) | v0.4 |
| WASM Compatibility | v0.5 |
