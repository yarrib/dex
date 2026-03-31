# Why dex?

dex competes in a space with existing scaffolding tools. Here's why it exists and when to use it.

---

## vs Cookiecutter

[Cookiecutter](https://cookiecutter.readthedocs.io) is the most common Python project scaffolding tool. It works, but it has a few rough edges for data engineering teams.

### No runtime required

Cookiecutter requires Python and `pip install cookiecutter`. On a new machine, in a CI job,
or on a team with mixed Python/non-Python backgrounds, this is friction. dex is a single
static binary — `curl | sh` and you're done.

### Org-wide variable pre-fill

Cookiecutter prompts for every variable, every time. dex has two mechanisms to skip prompts
for values your org already knows:

- **Standards** (`~/.config/dex/standards.toml`): flat key-value file that auto-fills any
  matching variable (e.g. `author`, `python_version`). Set once, never prompted again.
- **Presets** (`~/.config/dex/presets.toml`): named profiles for different project contexts
  (ML workloads vs ETL pipelines, dev workspace vs staging workspace).

For platform teams standardizing tooling across an org, this matters. You want to distribute
opinionated defaults without making every developer answer the same five questions every time.

### Beyond scaffolding

Cookiecutter stops at file generation. dex continues into the project lifecycle:

- `dex run <task>` — run tasks defined in `dex.toml` (tests, linting, deploy scripts)
- `dex db`, `dex az` — pass-through commands that proxy to external CLIs, configured in `dex.toml`
- `dex agent new` — scaffolds AI agent projects
- `dex mcp serve` — exposes dex tools to Claude and other MCP clients

One binary, one config file, from `init` through daily development.

### No template hosting required

Cookiecutter templates are typically GitHub repos that users clone. dex supports the same
pattern, but also embeds its built-in templates directly in the binary (zero network calls)
and allows teams to point at a local directory or a remote git repo via config — no separate
template registry service needed.

---

## vs Databricks Asset Bundle (DABs) Templates

Databricks Asset Bundles ship their own `databricks bundle init` scaffolding. It's Databricks'
own tool for their own format. It works well for pure Databricks projects, but it has a narrower
scope than dex by design.

### Generic, not Databricks-specific

DABs templates are built for one platform. Most data teams also maintain:

- Plain Python packages (shared utilities, libraries)
- AI agent projects (Claude, OpenAI, custom)
- Custom internal tools that don't fit the Databricks mold

dex uses the same template system, same prompt flow, and same config format for all of these.
One mental model for your entire project portfolio.

### Standards and presets don't exist in DABs init

`databricks bundle init` prompts you for values. dex's standards and presets layer lets you
pre-fill those values org-wide — including for Databricks-specific variables like
`workspace_url`, `cluster_id`, and `python_version`. Teams can ship a `standards.toml` with
their dex onboarding guide and skip the majority of prompts entirely.

### Org templates without forking Databricks tooling

DABs templates live inside the Databricks CLI codebase. If your org wants a custom template
that deviates from Databricks' defaults, you're maintaining a fork or working around it.
dex's org template model is first-class: point to a directory or a git repo in
`~/.config/dex/config.toml`, and `dex init` picks it up alongside built-in templates. No fork,
no separate tool, no special distribution mechanism.

---

## Why "generic" is the right default

It's tempting to build a narrowly scoped tool that does one thing perfectly. But for a
platform team, the cost of having five different scaffolding tools (one per project type)
is real: different conventions, different learning curves, different ways to extend.

dex's template model is intentionally generic:

- `template.toml` variables are just names and types — no Databricks-specific schema
- File rules are path patterns — works for any directory layout
- Standards and presets are flat key-value maps — applies to any variable in any template
- Pass-throughs are just subprocess delegation — wraps any CLI, not just Databricks

The built-in templates happen to target Databricks workflows because that's the primary user.
But an org running AWS Glue or dbt can write their own templates and dex becomes their tool too,
with no changes to the binary.

**The goal is a single, well-understood scaffolding and project operations convention — not
one that locks you into a specific platform's mental model.**
