# dex — Project Specification

> What uv did for Python packaging, dex does for Databricks project operations.

## 1. Overview

dex is an opinionated CLI framework for Databricks/MLOps project operations. It provides
scaffolding, task running, context switching, and deployment — with an extensible architecture
that lets teams build their own org-specific tooling on top.

**Core philosophy:**

- **Rust engine, Python surface.** Performance-critical operations (template rendering, file I/O,
  config parsing) live in Rust. The user-facing CLI is Python (click), making it trivially
  extensible by the teams who use it.
- **Opinionated defaults, escape hatches everywhere.** dex ships with strong opinions about
  project structure and workflows, but every opinion is overridable.
- **Pass-through, not reimplementation.** dex wraps existing CLIs (databricks, az, aws) rather
  than reimplementing their functionality. It adds ergonomics on top.
- **Framework, not just a tool.** Teams install dex as a dependency and build their own
  CLI (`acme-dex`, `myorg-ops`) on top. Like Flask for MLOps CLIs.

## 2. Target Users

- **ML engineers** working with Databricks who want consistent project structure and workflows.
- **Platform/MLOps teams** standardizing tooling across an organization.
- **Data engineers** building pipelines on Databricks who want ergonomic project operations.

The primary user is a team that wants to ship an internal CLI tool for their MLOps workflows
without building everything from scratch.

## 3. Distribution

dex is distributed as a Python package built with maturin. The Rust core is compiled into a
Python extension module via PyO3.

```bash
# Install dex itself
uv tool install dex

# Or as a dependency for building org-specific CLIs
uv add dex
```

Teams build on top of dex:

```bash
# A team's custom CLI built on dex
uv tool install acme-dex
acme-dex init --template ml-pipeline
acme-dex db clusters list
acme-dex deploy staging
```

## 4. CLI Interface

### 4.1. Built-in Commands

```
dex init [--template <name>] [--dir <path>] [--no-prompt]
    Scaffold a new project from a template. Prompts for variables interactively
    unless --no-prompt is set (uses defaults).

dex add <component> [--dry-run]                              # future
    Bolt a component onto an existing project. Components: ci, serving,
    monitoring, feature-store, etc.

dex run <task> [--parallel] [-- <extra-args>]                # future
    Run a task defined in dex.toml. Like make but with beautiful output,
    TOML config, and no runtime dependency.

dex switch <profile>                                         # future
    Switch workspace/environment context. Updates .databrickscfg, env vars,
    and any profile-aware config.

dex deploy [<target>] [--dry-run] [--promote]                # future
    Bundle and deploy. Wraps DABs, REST API calls, or custom deploy logic.
    --promote moves between environments (dev → staging → prod).

dex config [get|set|list]                                    # future
    Manage dex configuration.

dex self update                                              # future
    Update dex to the latest version.
```

### 4.2. Pass-through Commands

Pass-throughs delegate to external CLIs, forwarding all arguments:

```
dex db <args...>        →  databricks <args...>
dex az <args...>        →  az <args...>
```

Pass-throughs are configured in `dex.toml` or registered programmatically:

```toml
# dex.toml
[passthrough.db]
command = "databricks"
description = "Databricks CLI"

[passthrough.az]
command = "az"
description = "Azure CLI"
```

Pass-throughs are first-class: they appear in `dex --help`, support `--help` forwarding,
and can have pre/post hooks.

### 4.3. User-defined Commands

Teams add commands by writing Python:

```python
# In their package
import click
from dex.cli import create_cli

cli = create_cli()

@cli.command()
@click.argument("environment")
def promote(environment):
    """Promote current model to an environment."""
    ...
```

## 5. Configuration

### 5.1. Project Config: `dex.toml`

Lives at the project root. Defines project-level settings, tasks, and pass-throughs.

```toml
[project]
name = "my-ml-project"
description = "Revenue forecasting pipeline"
template = "ml-pipeline"           # template this project was scaffolded from

[passthrough.db]
command = "databricks"
description = "Databricks CLI"

[tasks.test]
command = "pytest tests/"
description = "Run tests"

[tasks.lint]
command = "ruff check ."
description = "Lint code"

[tasks.build]
command = "python -m build"
description = "Build package"
depends_on = ["lint", "test"]

[profiles.dev]
workspace_url = "https://dev.cloud.databricks.com"
cluster_id = "0123-456789-abcdef"

[profiles.staging]
workspace_url = "https://staging.cloud.databricks.com"
cluster_id = "9876-543210-fedcba"
```

### 5.2. User Config: `~/.config/dex/config.toml`

User-level defaults and preferences.

```toml
[defaults]
template = "ml-pipeline"
python_version = "3.11"

[templates]
# Additional template directories to search
paths = ["~/dex-templates"]

[ui]
color = "auto"    # auto | always | never
```

## 6. Template System

dex templates are **additive layers**. They can optionally declare a Databricks Asset
Bundle (DABs) template as a foundation, then add dex-specific files on top. This means
dex never reimplements DABs templating (which uses Go templates) — it delegates to
`databricks bundle init` and layers its opinions afterward.

### 6.1. Two Modes

**Standalone template** — dex renders everything itself (Jinja2 via minijinja):

```
dex init --template default
  → dex renders files/ through Jinja2 engine
  → writes to target directory
```

**DABs-composite template** — dex delegates the bundle scaffold to the Databricks CLI,
then layers dex-specific files on top:

```
dex init --template ml-pipeline
  Phase 1: databricks bundle init <dabs_source> --output-dir ./my-project
           → Go templates render → databricks.yml, notebooks, src/, etc.
           → DABs variables passed non-interactively via --config-file
  Phase 2: dex renders its own files/ on top
           → dex.toml, CI config, tasks, monitoring, etc.
           → Jinja2 templates, dex variables
           → will not overwrite files DABs already created (unless overwrite = true)
```

This composition model lets teams use **any existing DABs template** — official ones,
custom org templates from Git repos, local directories — and add dex opinions on top
without converting Go templates to Jinja2.

### 6.2. Template Structure

**Standalone** (dex owns the full scaffold):

```
my-template/
  template.toml              # manifest: metadata, variables, file rules
  files/                     # template files (Jinja2 syntax)
    pyproject.toml.j2
    README.md.j2
    src/
      {{ project_name }}/
        __init__.py
        main.py.j2
    tests/
      test_main.py.j2
    databricks.yml.j2
  hooks/                     # lifecycle hooks (Python scripts)
    post_scaffold.py
```

**DABs-composite** (dex adds on top of a DABs template):

```
my-template/
  template.toml              # manifest — includes [template.dabs] section
  files/                     # ONLY dex-specific additions
    dex.toml.j2              # dex project config
    .github/
      workflows/
        ci.yml.j2            # CI config that DABs doesn't provide
    Makefile.j2
  hooks/
    post_scaffold.py
```

When a DABs base is declared, `files/` contains only what dex **adds** — not the
full project. DABs provides the bundle scaffold (databricks.yml, notebooks, src/).
dex provides the ops layer (dex.toml, CI, tasks, monitoring).

### 6.3. Template Manifest: `template.toml`

#### Standalone (no DABs base)

```toml
[template]
name = "default"
description = "Minimal Databricks ML project"
version = "0.1.0"
min_dex_version = "0.1.0"

[[variables]]
name = "project_name"
prompt = "Project name"
type = "string"
required = true
validate = "^[a-z][a-z0-9_-]*$"

[[variables]]
name = "python_version"
prompt = "Python version"
type = "choice"
choices = ["3.10", "3.11", "3.12"]
default = "3.11"
```

#### DABs-composite

```toml
[template]
name = "ml-pipeline"
description = "Databricks ML pipeline with full ops tooling"
version = "0.1.0"
min_dex_version = "0.1.0"

# DABs template as foundation — rendered by `databricks bundle init`
[template.dabs]
# Any valid source for `databricks bundle init`:
source = "https://github.com/databricks/mlops-stacks"
#   source = "default-python"                    # built-in DABs template name
#   source = "/path/to/local/dabs/template"      # local path
#   source = "https://github.com/myorg/templates" # Git URL

# How DABs variables are prompted. One of:
#   "passthrough" — let `databricks bundle init` run interactively (default)
#   "unified"     — dex reads databricks_template_schema.json from the source,
#                    merges with dex variables, presents one prompt flow
#   "mapped"      — pre-fill via variable_map, DABs prompts for the rest
prompt = "unified"

# For "mapped" mode only: map dex variable names → DABs variable names.
# [template.dabs.variable_map]
# project_name = "input_project_name"

# For "unified" mode: override specific DABs schema variables.
# dex reads the schema automatically — these overrides let you change
# defaults, restrict choices, or hide variables.
[template.dabs.overrides.input_cloud]
default = "azure"

# dex-specific variables (beyond what DABs provides)
[[variables]]
name = "include_ci"
prompt = "Include CI/CD configuration?"
type = "bool"
default = true

[[variables]]
name = "monitoring_tier"
prompt = "Monitoring tier"
type = "choice"
choices = ["basic", "full"]
default = "basic"

# Conditional file inclusion (dex layer only)
[[files]]
src = ".github/"
condition = "include_ci"

# File path remapping
[[files]]
src = "cloud/{{ cloud_provider }}/"
dest = "infra/"
```

### 6.4. DABs Prompt Modes

The `prompt` field in `[template.dabs]` controls how DABs template variables are
collected. There are three modes, each with different tradeoffs:

#### Mode 1: `"passthrough"` (default)

Let `databricks bundle init` handle its own interactive prompts. dex prompts for
its own variables afterward. Two prompt sessions, zero configuration.

```
dex init --template ml-pipeline
    │
    ▼  Phase 1 — interactive DABs prompts
    databricks bundle init <source> --output-dir <target>
      → "Project name [my_project]: " (from DABs schema)
      → "Cloud provider (aws/azure/gcp) [azure]: " (from DABs schema)
      → ... (all DABs prompts run natively)
    │
    ▼  Phase 2 — dex prompts
    dex: "Include CI/CD configuration? [Y/n]: "
    dex: "Monitoring tier (basic/full) [basic]: "
    │
    ▼  Phase 3 — dex renders its own files on top
```

**Use when:** You want simplicity, or the DABs template has complex conditional
prompts (`skip_prompt_if`) that are hard to replicate.

#### Mode 2: `"unified"` (best UX)

dex fetches the `databricks_template_schema.json` from the template source,
parses its properties, merges with dex variables, and presents **one unified
prompt flow**. All prompts come through dex's click-based UI.

```
dex init --template ml-pipeline
    │
    ▼  Single prompt flow (dex handles everything)
    "Project name [my_project]: "          ← from DABs schema
    "Cloud provider (aws/azure/gcp): "     ← from DABs schema, override default
    "Include CI/CD configuration? [Y/n]: " ← from dex template.toml
    "Monitoring tier (basic/full): "        ← from dex template.toml
    │
    ▼  Phase 1 — non-interactive DABs scaffold
    databricks bundle init <source> \
      --output-dir <target> \
      --config-file <full-config.json>   ← all DABs vars pre-filled
    │
    ▼  Phase 2 — dex renders its own files on top
```

How dex resolves the schema:

1. **Local path** — read `<source>/databricks_template_schema.json` directly
2. **Git URL** — shallow-clone to temp dir, read schema, pass clone path to
   `databricks bundle init` (avoids cloning twice)
3. **Built-in name** — not supported for unified mode (use passthrough)

The `[template.dabs.overrides]` section lets template authors customize DABs
prompts: change defaults, restrict choices, rewrite descriptions. Properties not
overridden use the values from the DABs schema as-is.

**Use when:** You want the best UX — one prompt flow, consistent styling, and
the ability to customize DABs prompts for your org.

#### Mode 3: `"mapped"`

Pre-fill specific DABs variables via `variable_map`, let `databricks bundle init`
prompt interactively for anything else. A hybrid of passthrough and unified.

```toml
[template.dabs]
source = "https://github.com/databricks/mlops-stacks"
prompt = "mapped"

[template.dabs.variable_map]
project_name = "input_project_name"    # dex var → DABs var
cloud_provider = "input_cloud"
```

```
dex init --template ml-pipeline
    │
    ▼  dex prompts for its own variables
    "Project name: "        ← dex variable (also mapped to DABs)
    "Cloud provider: "      ← dex variable (also mapped to DABs)
    "Include CI? [Y/n]: "   ← dex-only variable
    │
    ▼  Phase 1 — partially non-interactive DABs scaffold
    databricks bundle init <source> \
      --output-dir <target> \
      --config-file <partial-config.json>   ← only mapped vars
      → DABs prompts for unmapped vars (if any)
    │
    ▼  Phase 2 — dex renders its own files on top
```

**Use when:** You want to share some variables between dex and DABs (e.g.
`project_name`) without reading the full schema, and you're OK with the
DABs CLI prompting for anything you didn't map.

### 6.5. `databricks_template_schema.json` Parsing

For unified mode, dex parses the DABs schema and converts properties to dex
variable specs:

| DABs Schema Field | dex Variable Field | Notes                          |
|-------------------|--------------------|--------------------------------|
| property name     | `name`             | Direct mapping                 |
| `description`     | `prompt`           | Used as prompt text            |
| `type`            | `var_type`         | `"string"` → `String`/`Choice` |
| `default`         | `default`          | Supports `{{.other_var}}` refs |
| `enum`            | `choices`          | Converts type to `Choice`      |
| `pattern`         | `validate`         | Regex validation               |
| `order`           | sort key           | Lower = prompted first         |
| `skip_prompt_if`  | (evaluated)        | Conditional prompting          |

DABs variables are prompted first (sorted by `order`), followed by dex-specific
variables (sorted by declaration order in `template.toml`). Variable names that
appear in both DABs schema and dex `[[variables]]` are deduplicated — the dex
definition takes precedence (allowing the template author to override prompts,
defaults, or validation).

### 6.6. Variable Types

| Type     | Rust Type | Prompt Widget           |
|----------|-----------|-------------------------|
| `string` | `String`  | Text input              |
| `bool`   | `bool`    | Confirm (y/n)           |
| `choice` | `String`  | Select from list        |
| `multi`  | `Vec`     | Multi-select from list  |

### 6.7. Template Sources

Templates are resolved in order:

1. **Embedded** — built-in templates compiled into the binary
2. **Project-local** — `./templates/` directory
3. **User-configured** — paths in `~/.config/dex/config.toml`
4. **Python entry points** — templates registered by installed packages

### 6.8. Hooks

Hooks are Python scripts invoked at lifecycle points:

- `pre_scaffold` — before any files are written
- `post_scaffold` — after all files are written

Hook scripts receive a context object with variables, paths, and the dex API:

```python
# hooks/post_scaffold.py
def run(ctx):
    ctx.run("uv init")
    ctx.run("git init")
    ctx.log("Project scaffolded successfully!")
```

## 7. Extension Model

dex is designed as a framework. Teams build org-specific CLIs on top.

### 7.1. Creating an Org CLI

```python
# acme_dex/cli.py
from dex.cli import create_cli, passthrough

cli = create_cli(
    name="acme-dex",
    templates_dir="./templates",       # org templates
    config_defaults={
        "cloud_provider": "azure",
    },
    passthroughs=[
        passthrough("db", command="databricks", description="Databricks CLI"),
        passthrough("az", command="az", description="Azure CLI"),
    ],
)

@cli.command()
@click.argument("environment")
def promote(environment):
    """Promote model to an environment."""
    ...
```

```toml
# pyproject.toml
[project]
name = "acme-dex"
dependencies = ["dex"]

[project.scripts]
acme-dex = "acme_dex.cli:cli"
```

```bash
uv tool install acme-dex
acme-dex init --template ml-pipeline
acme-dex promote staging
```

### 7.2. Plugin Entry Points

Packages can register templates and commands via entry points without
subclassing the CLI:

```toml
# pyproject.toml
[project.entry-points."dex.templates"]
ml-pipeline = "acme_dex.templates:ml_pipeline"

[project.entry-points."dex.commands"]
promote = "acme_dex.commands:promote"
```

### 7.3. Rust Core API (via PyO3)

The Python bindings expose dex-core's functionality:

```python
from dex._core import (
    render_template,       # render a Jinja2 string with variables
    scaffold_project,      # full scaffold operation
    parse_template_manifest,  # parse a template.toml
    load_config,           # parse and merge dex.toml configs
)
```

The `_core` module is the FFI boundary. The public Python API (`dex.*`) wraps
it with Pythonic interfaces.

## 8. Agent Scaffolding (`dex agent`)

### 8.1. Overview

`dex agent` extends dex with opinionated agent project scaffolding. It combines
deterministic project generation with a generative Q&A flow powered by Claude to
produce a working, deployable agent project — fully integrated into the DAB
workflow dex already manages.

Agent projects are just DABs. Everything dex does (deploy, validate, pass-through)
works on agent projects without special casing.

```
dex agent new              # scaffold a new agent project
dex agent eval             # run evals and log results to MLflow (future)
dex agent add tool         # scaffold a tool into an existing project (future)
dex deploy                 # existing dex DAB deploy — works unchanged
```

### 8.2. Philosophy

- **Opinions are the value.** MLflow tracing, structured logging, evals, and DAB
  config are always included. Remove them if you don't want them — but you'll
  never have to add them.
- **Scaffold fast, build inside structure.** Deterministic generation creates the
  project. Claude fills in agent logic inside that structure.
- **Describe behavior, not boilerplate.** The Q&A asks what the agent does, not
  how to wire up infrastructure.

### 8.3. Q&A Flow

The interactive flow collects just enough context to scaffold meaningfully:

1. **What does this agent do in one sentence?**
2. **What triggers it?** (user request / schedule / event / upstream system)
3. **What does success look like?**
4. **What does it need to read?** (tables, APIs, files)
5. **What does it need to write or change?**
6. **Does it hand off to a human or another agent?**
7. **Autonomous or confirm before acting?**
8. **Example input and correct output?**
9. **What would a bad or dangerous output look like?**
10. **Job, serving endpoint, or interactive?**

Answers drive:
- The `CLAUDE.md` and system prompt
- Tool stubs
- The first eval test case
- DAB config (job vs serving endpoint, schedule, etc.)

### 8.4. Generated Structure

```
my-agent/
├── CLAUDE.md                   # project instructions for Claude Code
├── databricks.yml              # DAB root config
├── resources/
│   └── my_agent_job.yml        # DAB job or serving endpoint definition
├── src/
│   └── my_agent/
│       ├── __init__.py
│       ├── agent.py            # agent definition and loop
│       ├── tools/
│       │   ├── __init__.py     # tool discovery and ToolResult dataclass
│       │   └── example_tool.py # stub tool(s) from Q&A
│       ├── prompts/
│       │   └── system.md       # system prompt from Q&A
│       └── tracing.py          # MLflow tracing (always included)
├── evals/
│   ├── runner.py               # eval harness → MLflow
│   └── cases/
│       └── example.json        # one eval case from Q&A
├── tests/
│   └── test_agent.py
├── pyproject.toml
└── .env.example
```

### 8.5. Baked-In Defaults

**MLflow Tracing** — every agent run is traced. Experiment name derived from
project name. Traces include input, output, tool calls, latency, errors.

**Structured Logging** — JSON logging only. No print statements in generated code.

**Evals Folder** — always generated with a runner and one seeded test case.

**CLAUDE.md** — always generated. Contains agent description, project structure,
conventions, and constraints for Claude Code.

**DAB Config** — always generated and deployable.

**Tool Interface** — all tools return `ToolResult(success, data, error)` and are
auto-discovered from the `tools/` directory.

### 8.6. Two-Phase Generation

**Phase 1 — Deterministic (Rust core).** File structure, DAB config, tracing
setup, tool interface, eval harness. Fast, predictable, testable.

**Phase 2 — Generative (Python layer → Claude API).** Flesh out `agent.py`,
complete tool stubs, write the system prompt, fill in the eval case. Claude
operates inside the already-generated structure. The `CLAUDE.md` constrains it.

The generative phase is optional (`--no-generate` flag) — you can scaffold
the structure and write the agent logic yourself.

### 8.7. Eval Case Format

```json
{
    "id": "example-01",
    "description": "Basic happy path",
    "input": "...",
    "expected_behavior": "...",
    "should_not": "..."
}
```

Results are logged to the MLflow experiment as a run with pass/fail metrics.

## 9. v0.1 Scope

**Ship: `dex init` with one built-in template.**

### What's In

- `dex init` command with interactive prompts
- One hardcoded "default" template for Databricks ML projects
- Template rendering via minijinja (Jinja2 syntax)
- `template.toml` manifest format with variable declarations
- Beautiful terminal output (colors, spinners, styled prompts)
- `dex.toml` project config (project section only)
- Python package installable via `uv pip install` / `uv tool install`
- `create_cli()` factory for building org CLIs on top

### What's Out (Future)

- `dex add`, `dex run`, `dex switch`, `dex deploy`
- Pass-through commands
- Template registry / multiple templates
- User config (`~/.config/dex/config.toml`)
- Plugin entry points
- Hooks
- Conditional file inclusion
- `dex self update`

### Week-by-Week (v0.1)

- **Week 1**: Rust core — template.toml parsing, minijinja rendering, file scaffolding.
  PyO3 bindings. `dex init` with hardcoded default template.
- **Week 2**: TOML-driven template registry. Variable validation. Click CLI with
  `create_cli()` factory.
- **Week 3**: Variable interpolation in file paths. Polish terminal output. Package
  for distribution via maturin + uv.
