# PRD: Templating Strategy

**Status:** Decided
**Target:** v0.2
**Owner:** TBD

---

## Decision

All templates are standalone. dex renders every file via minijinja. There is no
dependency on `databricks bundle init` or any external CLI at scaffold time.

The composite mode concept (`[template.dabs]`, `DabsBaseSpec`, `dabs_schema.rs`) has
been removed entirely. The DABs-specific templates (`dabs-etl`, `dabs-ml`,
`dabs-aiagent`) own their full file trees and render them the same way every other
template does.

---

## Rationale

- Composite mode was never implemented — the wiring from `[template.dabs]` to an
  actual `databricks bundle init` subprocess never existed beyond dead structs.
- Requiring the Databricks CLI on PATH at scaffold time adds an unnecessary dependency
  and breaks `dex init` in CI, fresh environments, and Snowflake/non-Databricks contexts.
- dex templates are already richer than what DABs generates: typed variable specs,
  conditional file inclusion, Jinja2 filters, and hooks. Delegating Phase 1 to
  `bundle init` adds complexity without adding capability.
- Maintaining our own `databricks.yml` stubs is straightforward — the format is
  well-documented and changes slowly.

---

## What was removed

| Item | Location | Disposition |
|------|----------|-------------|
| `DabsBaseSpec` | `manifest.rs` | Deleted |
| `DabsPromptMode` | `manifest.rs` | Deleted |
| `DabsVariableOverride` | `manifest.rs` | Deleted |
| `dabs` field on `TemplateMetaRaw` | `manifest.rs` | Deleted |
| `dabs_schema.rs` | `crates/dex-core/src/template/` | Deleted |
| `parse_dabs_composite_manifest` test | `manifest.rs` | Deleted |

---

## v0.2 implementation steps

1. **Fix `VariableSpec` prompting** in `dex-cli`:
   - Loop over all `template.variables`, prompt for each with type/default/validation
   - Pass full `variables` map to `scaffold()`

2. **Fix `include_dir` fallback** in `registry.rs`:
   - Remove the dead first-pattern branch; keep only the working one

3. **Flesh out standalone DABs templates** (`dabs-etl`, `dabs-ml`, `dabs-aiagent`):
   - Each owns its complete `databricks.yml`, `pyproject.toml`, and source stubs
   - See `prd-templates.md` for per-template file trees
