# dex — Scope Guardrails

This document is the canonical reference for what dex is and is not. It exists to prevent
scope creep and to give contributors, maintainers, and product discussions a shared filter
for evaluating new features.

When in doubt about whether something belongs in dex, consult this document first.

---

## 1. Purpose Statement

dex is a **data project operations CLI**. It standardizes how ML engineers and platform
teams create, configure, and run their projects — particularly on Databricks. It does this
through three mechanisms: scaffolding structure from templates, running project-local tasks,
and syncing project tooling configuration.

dex helps teams **set up and operate** their projects. It does not build, deploy, or run
the projects themselves — those responsibilities belong to the tools dex wraps.

The CLI form factor is deliberate: a single native binary, no runtime, no server, no
dependencies. dex installs in one command and works anywhere.

---

## 2. Target Users

dex is built for:

- **ML engineers** working with Databricks who want consistent project structure and workflows
- **Platform / MLOps teams** standardizing tooling across an organization
- **Data engineers** building pipelines who want ergonomic project operations

dex is **not** a general-purpose developer tool. Features that only make sense outside
the data/ML/Databricks context are out of scope.

---

## 3. The Three Primitives

Every feature in dex must map to one of three operations:

| Primitive | What it does | Examples |
|-----------|--------------|---------|
| **Scaffold** | Create project structure from templates | `dex init`, `dex agent new`, `dex add` |
| **Run** | Execute project-local tasks or delegate to external CLIs | `dex run`, pass-throughs |
| **Sync** | Keep project tooling configuration up to date | `dex skills sync` |

A proposed feature that doesn't map cleanly to Scaffold, Run, or Sync is out of scope
by default. Adding a fourth primitive requires a SPEC.md revision and explicit team
discussion — not just an implementation.

---

## 4. In-Scope Boundaries

The following are explicitly in scope, with rationale:

| Area | Rationale |
|------|-----------|
| Project scaffolding from templates (minijinja) | Core primitive — the original reason dex exists |
| Task running (project-local, defined in `dex.toml`) | Ergonomic wrapper around project-specific commands |
| Pass-through delegation to external CLIs | Ergonomics — one entry point, not reimplementation |
| AI agent scaffolding (`dex agent new`) | Agents are a first-class data project artifact in the Databricks ecosystem |
| MCP server (`dex mcp serve`) | Exposes dex's existing operations to AI tools; adds no new logic |
| Skill pack management (`dex skills`) | Scaffolds and syncs AI coding configs *for the project*, not for dex itself |
| Template discovery (`dex templates`) | Introspection of the Scaffold primitive |
| Composable traits (`dex add`) | Template-based composition — Scaffold extended |

---

## 5. Anti-Features

The following will **never** be built into dex. Each has a better dedicated tool.

| Anti-feature | Use instead |
|---|---|
| Deployment orchestration | Databricks Asset Bundles, Terraform, Pulumi |
| Dependency / package management | uv, poetry, pip, cargo |
| CI/CD pipeline management | GitHub Actions, GitLab CI, Buildkite |
| Monitoring / observability | Databricks, Grafana, Datadog |
| Secrets management | Vault, AWS Secrets Manager, Azure Key Vault |
| Container / image building | Docker, Buildkit, ko |
| Code execution runtime | dex invokes commands; it does not run code |
| Plugin code system | Extensibility is config + templates only; no dynamic loading |
| Hosted / cloud service | dex is a local CLI, not a SaaS |
| General software project tooling | Out of target user scope |

If a request maps to one of these categories, the answer is: delegate to the right tool,
and if ergonomics are needed, add a pass-through in `dex.toml`.

---

## 6. The Scope Decision Filter

Before implementing a new feature, answer all six questions. Every answer must be **yes**.

1. **User fit** — Does this serve ML engineers or platform teams on data projects specifically?
2. **Primitive fit** — Does it map to Scaffold, Run, or Sync?
3. **Delegation over reimplementation** — Does it wrap or delegate to existing tools rather than reimplement their functionality?
4. **Local-first** — Does it stay local? (Network calls are acceptable only for fetching templates or skill packs.)
5. **Config gap** — Would removing it leave a gap that config or templates alone can't fill?
6. **Binary discipline** — Does it avoid inflating the binary with non-essential logic or large dependencies?

If any answer is **no**, the feature is out of scope. A compelling argument for an
exception requires updating this document, not bypassing it.

---

## 7. Scope Evolution Policy

The scope can grow, but deliberately:

- **New primitives** (beyond Scaffold / Run / Sync) require a SPEC.md revision and explicit
  team discussion before implementation begins.
- **New commands** must pass the decision filter above; additions should be documented in SPEC.md.
- **Features listed as "future" in SPEC.md** require explicit promotion (a SPEC.md update
  and team sign-off) — not just an implementation appearing on a branch.
- **New dependencies** must be justified in the dependency table in ARCHITECTURE.md.
  The bar is: essential, well-maintained, and not reimplementable in a few lines.
- **Pass-throughs are the default answer** to "dex should support X" — configure it in
  `dex.toml` before proposing a new built-in command.

---

## 8. Mono-Repo Expansion Policy

Additional delivery surfaces (wasm build, web UI, browser extension, etc.) are permitted
within this repository if — and only if — they are fully decoupled from the primary build:

- They live in a separate crate or top-level directory (e.g., `crates/dex-wasm/`, `web/`)
- `cargo build` and `cargo test` at the workspace root are **not** affected
- They may use `dex-core` as a library but must not add business logic to it
- They are excluded from the default CI matrix unless explicitly opted in
- `dex-cli` must not depend on them

The primary binary stays lean. A wasm or web surface that reuses `dex-core` is a valid
expansion; one that forces `dex-cli` to carry new weight is not.

---

## 9. Current Feature Verdicts

| Feature | Primitive | Verdict | Notes |
|---|---|---|---|
| `dex init` | Scaffold | In scope | Core feature |
| `dex run` | Run | In scope | Project-local tasks only; not a general task runner |
| Pass-throughs | Run | In scope | Delegation, not reimplementation |
| `dex agent new` | Scaffold | In scope | Agents are data project artifacts |
| `dex mcp serve` | Run | In scope | Exposes dex ops; adds no new logic |
| `dex add` | Scaffold | In scope | Composable traits via templates |
| `dex skills` | Sync | In scope | Syncs AI coding configs for the project, not for dex |
| `dex templates` | Scaffold | In scope | Template discovery and introspection |

**Watch list** (valid today, but must not expand beyond their current scope):

- `dex run` — must stay project-local. Global task runners, DAG engines, retry logic, and
  conditional execution belong in dedicated tools (Make, Just, Prefect, Airflow).
- `dex skills` — must stay at install/sync of markdown skill files. Version resolution,
  dependency graphs, and execution are out of scope.
- `dex agent new` — scaffolding only. Agent deployment, execution, monitoring, and
  observability belong in the data platform.
