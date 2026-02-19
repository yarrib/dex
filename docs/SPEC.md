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

Templates are directories containing a manifest (`template.toml`), template files, and
optional hooks. They are rendered using Jinja2 syntax (via minijinja in Rust).

### 6.1. Template Structure

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

### 6.2. Template Manifest: `template.toml`

```toml
[template]
name = "ml-pipeline"
description = "Databricks ML pipeline project"
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

[[variables]]
name = "include_ci"
prompt = "Include CI/CD configuration?"
type = "bool"
default = true

[[variables]]
name = "cloud_provider"
prompt = "Cloud provider"
type = "choice"
choices = ["azure", "aws", "gcp"]

# Conditional file inclusion
[[files]]
src = ".github/"
condition = "include_ci"

# File path remapping
[[files]]
src = "cloud/{{ cloud_provider }}/"
dest = "infra/"
```

### 6.3. Variable Types

| Type     | Rust Type | Prompt Widget           |
|----------|-----------|-------------------------|
| `string` | `String`  | Text input              |
| `bool`   | `bool`    | Confirm (y/n)           |
| `choice` | `String`  | Select from list        |
| `multi`  | `Vec`     | Multi-select from list  |

### 6.4. Template Sources

Templates are resolved in order:

1. **Embedded** — built-in templates compiled into the binary
2. **Project-local** — `./templates/` directory
3. **User-configured** — paths in `~/.config/dex/config.toml`
4. **Python entry points** — templates registered by installed packages

### 6.5. Hooks

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

## 8. v0.1 Scope

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
