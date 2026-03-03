# PRD: Templating Strategy

**Status:** Planning
**Target:** v0.2 (fixes) / v0.3 (unified mode)
**Owner:** TBD

---

## 1. Current State + Dead Code Audit

### What works

- `dex init --template default` scaffolds files from embedded templates.
- `list_embedded_templates()` returns template metadata.
- `scaffold_project()` renders Jinja2 files via minijinja and writes them.
- Variable spec parsing (`VariableSpec`) is implemented in `dex-core`.

### Dead code / broken paths

| Item | Location | Status |
|------|----------|--------|
| `_run_dabs_init` | `cli.py:163-217` | Defined, never called |
| `dabs_schema.rs` | `crates/dex-core/src/` | Module exists, no callers |
| `FileRule.overwrite` | `scaffold.rs` | Field parsed, never checked |
| `VariableSpec` prompting | `cli.py:132-143` | Only `project_name` prompted; rest ignored |
| `include_dir` fallback | `registry.rs:125-131` | First pattern always fails |

These are known issues. This document tracks the strategy for fixing them,
not the bugs themselves (see bug-fix branch).

---

## 2. DABs `bundle init` Go-Template System

Databricks Asset Bundles use a Go template system (`databricks bundle init <template-url>`).

### Pros
- First-class Databricks support — maintained by Databricks
- Rich existing templates (default-python, mlops-stacks, etc.)
- Handles Databricks-specific variable prompting and validation
- `--config-file` flag enables non-interactive use

### Cons
- Go template syntax is unfamiliar to Python/MLOps users
- No Jinja2 — users can't reuse existing knowledge
- Cannot be extended from Python
- Variables are DABs-specific — no way to inject dex-level variables
- Template URLs require network access (or local clone)
- No programmatic API — subprocess only

---

## 3. Custom minijinja (Current Approach)

dex-core uses [minijinja](https://github.com/mitsuhiko/minijinja) (Rust) to render
`.j2` template files with Jinja2-compatible syntax.

### Pros
- Jinja2 syntax — familiar to Python users
- Embedded at compile time via `include_dir` — zero runtime filesystem dependency
- Fully testable in Rust without subprocess
- Variable spec is structured (type, default, validation, prompt text)
- Extensible: custom filters, globals, template inheritance

### Cons
- We maintain the templates (DABs `bundle init` handles this for DABs templates)
- No existing ecosystem of Databricks-specific templates
- Must replicate what DABs would generate for DABs projects

---

## 4. Composite Mode: Design Analysis

Composite mode was designed to let dex delegate Phase 1 to `databricks bundle init`
(for DABs boilerplate) and overlay Phase 2 files on top (for dex-specific additions).

### What's broken

1. `_run_dabs_init` is never called. The code path doesn't exist.
2. `dabs_schema.rs` parses DABs schemas but has no callers.
3. `FileRule.overwrite` is never checked, so Phase 2 cannot safely overlay Phase 1 files.
4. The `[template.dabs]` section in `template.toml` is parsed but has no effect.

### What the design intends

```
dex init --template dabs-etl --dir ./my-project
    → Phase 1: databricks bundle init default-python --config-file /tmp/dex-dabs-xxx.json
              (writes DABs boilerplate to ./my-project/)
    → Phase 2: dex scaffold overlay
              (writes/merges dex-specific files, respecting FileRule.overwrite)
```

### Design issues

- Phase 1 requires `databricks` CLI on PATH. This should be detected early with
  a clear error, not at scaffold time.
- The config JSON for `databricks bundle init --config-file` must map dex variable
  names to DABs variable names. The `variable_map` field handles this.
- Phase 2 overlay needs `FileRule.overwrite` to be respected — currently always overwrites.

---

## 5. Recommendation

### Support both systems; keep them separate

| Template type | Rendering | When to use |
|--------------|-----------|-------------|
| Standalone | minijinja (dex-core) | Pure Python packages, simple DABs bundles |
| DABs-composite | Phase 1 → bundle init, Phase 2 → minijinja | Complex DABs templates that need Databricks-native scaffolding |

### Defer unified prompt mode to v0.3

A "unified" prompt mode that merges DABs and dex variable prompts into a single
interactive session requires:
- Parsing DABs template schemas from the network (or cache)
- Merging two prompt trees
- Handling DABs conditional variables

This is v0.3 scope. For v0.2: fix `passthrough` mode, fix `_run_dabs_init` wire-up,
fix `FileRule.overwrite`.

### Exact implementation steps for v0.2

1. **Fix `VariableSpec` prompting** (`cli.py:init_command`):
   - Call `parse_template_manifest()` to get all variables
   - Loop over `template.variables`, prompt for each with type/default/validation
   - Pass full `variables` dict to `scaffold_project()`

2. **Wire `_run_dabs_init`** into `init_command`:
   - Check if template manifest has `[template.dabs]`
   - If yes: run Phase 1 (`_run_dabs_init`), then Phase 2 (overlay)
   - If no: run standalone scaffold (current behavior)

3. **Fix `FileRule.overwrite`** in `scaffold.rs`:
   - Check the field before writing
   - If `overwrite = false` and file exists, skip with a log message

4. **Fix `include_dir` fallback** in `registry.rs`:
   - Remove dead first-pattern branch
   - Keep only the working second pattern

5. **Add DABs mode detection** to `list_embedded_templates()`:
   - Return whether each template is standalone or composite
   - Surface this in `dex init` output
