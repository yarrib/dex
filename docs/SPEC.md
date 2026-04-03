# dex — Project Specification

## 1. Overview

dex is an opinionated CLI framework for data project operations. It provides scaffolding,
task running, and pass-through delegation — with an extensible architecture that lets
teams configure their own org-specific tooling on top.

**Core philosophy:**

- **100% Rust, single binary.** No Python runtime. No runtime dependencies. Ships as a
  self-contained native binary.
- **Opinionated defaults, escape hatches everywhere.** dex ships with strong opinions
  about project structure and workflows, but every opinion is overridable.
- **Pass-through, not reimplementation.** dex wraps existing CLIs (databricks, az, aws)
  rather than reimplementing their functionality. It adds ergonomics on top.
- **Config-driven extensibility.** Teams extend dex through `dex.toml` (pass-through
  commands) and custom template directories. No code required.

## 2. Target Users

- **ML engineers** working with Databricks who want consistent project structure and workflows.
- **Platform/MLOps teams** standardizing tooling across an organization.
- **Data engineers** building pipelines on Databricks who want ergonomic project operations.

## 3. Distribution

dex is distributed as a pre-built native binary.

```bash
# Install via script (Linux/macOS)
curl -sSf https://raw.githubusercontent.com/yarrib/dex/main/install.sh | sh

# Or build from source
cargo install --path crates/dex-cli
```

Teams extend dex by:
- Adding a custom `dex.toml` with pass-through commands
- Pointing to a custom templates directory
- Optionally wrapping dex in their own binary with a custom name

## 4. CLI Interface

### 4.1. Built-in Commands

```
dex init [--template <name>] [--dir <path>] [--no-prompt]
         [--preset <profile>] [--presets-file <path>]
         [--standards <path>]
    Scaffold a new project from a template. Prompts for variables interactively
    unless --no-prompt is set (uses defaults). --preset loads a named profile
    from the presets file; --standards loads flat key-value pre-fills.

dex init --template agent-anthropic
dex init --template agent-openai
dex init --template agent-baml
    Scaffold an AI agent project using a framework-specific template.
    Agent templates include: prompts, tools, evals, DAB deployment config.
    Choose the template matching your LLM SDK.

dex mcp serve
    Start the MCP server for AI agent integration.

dex run <task> [-- <extra-args>]
    Run a task defined in [tasks.*] in dex.toml. Respects depends_on ordering.
    Extra args after -- are appended to the task command.

dex add <component> [--dry-run]                              # future
    Bolt a component onto an existing project.

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

Pass-throughs are configured in `dex.toml`:

```toml
# dex.toml
[passthrough.db]
command = "databricks"
description = "Databricks CLI"

[passthrough.az]
command = "az"
description = "Azure CLI"
```

Pass-throughs appear in `dex --help` and support `--help` forwarding.

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
template = "dabs-package"

[templates]
# Additional template directories to search
paths = ["~/dex-templates"]

[ui]
color = "auto"    # auto | always | never
```

### 5.3. Presets: `~/.config/dex/presets.toml`

Named profiles of variable pre-fills. Select a profile with `dex init --preset <name>`;
any matching template variables are filled in without prompting.

```toml
[profiles.ml-project]
workspace_url  = "https://ml.cloud.databricks.com"
cluster_id     = "0123-456789-ml"
python_version = "3.12"

[profiles.etl]
workspace_url  = "https://etl.cloud.databricks.com"
python_version = "3.11"
```

```bash
dex init --template dabs-package --preset ml-project
```

**Pre-fill precedence (lowest → highest):**

1. Template defaults (`template.toml`)
2. Preset profile values (`--preset`)
3. Standards values (`--standards` / `~/.config/dex/standards.toml`)
4. Interactive prompt answer

### 5.4. Standards: `~/.config/dex/standards.toml`

Flat key-value file. Pre-fills template variables org-wide.

```toml
author         = "yarrib"
python_version = "3.12"
```

```bash
dex init --template default --standards ./org-standards.toml
```

## 6. Template System

dex templates are rendered by minijinja (Jinja2 syntax) in Rust. Built-in templates
are embedded in the binary at compile time.

### 6.1. Template Structure

```
my-template/
  template.toml              # manifest: metadata, variables, file rules
  files/                     # template files (Jinja2 syntax)
    pyproject.toml.j2
    README.md.j2
    dex.toml.j2
    src/
      {{ project_name }}/
        __init__.py
        main.py.j2
    tests/
      test_main.py.j2
```

### 6.2. Template Manifest: `template.toml`

```toml
[template]
name = "default"
description = "Minimal Python project"
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
choices = ["3.12", "3.11"]
default = "3.12"

[[files]]
src = "notebooks/"
condition = "include_notebook"
```

### 6.3. Variable Types

| Type     | Prompt Widget           |
|----------|-------------------------|
| `string` | Text input              |
| `bool`   | Confirm (y/n)           |
| `choice` | Select from list        |
| `multi`  | Multi-select from list  |

### 6.4. Template Sources

Templates are resolved in order:

1. **Embedded** — built-in templates compiled into the binary
2. **Project-local** — `./templates/` directory
3. **User-configured** — paths in `~/.config/dex/config.toml`

## 7. Extension Model

dex is extensible via config and templates — no code required.

### 7.1. Pass-through Commands via dex.toml

Configure pass-throughs in any `dex.toml`:

```toml
[passthrough.db]
command = "databricks"
description = "Databricks CLI"

[passthrough.tf]
command = "terraform"
description = "Terraform"
```

```bash
dex db clusters list
dex tf plan
```

### 7.2. Custom Templates

Add custom templates to a directory and reference it in `~/.config/dex/config.toml`:

```toml
[templates]
paths = ["~/acme-dex-templates"]
```

Or point dex at a directory directly:

```bash
dex init --template-dir ./templates --template my-template --dir my_project
```

See [Template Authoring](templates/authoring.md) and [Org Templates](templates/org-templates.md).

## 8. Agent Scaffolding (templates)

### 8.1. Overview

AI agent projects are scaffolded via `dex init` using framework-specific templates.
Choose the template that matches your LLM SDK. All agent templates share the same
variable set (description, trigger, reads/writes, etc.) and produce a deployable
skeleton integrated with Databricks Asset Bundles.

```
dex init --template agent-anthropic   # Anthropic SDK (claude-*)
dex init --template agent-openai      # OpenAI SDK (gpt-*)
dex init --template agent-baml        # BAML typed functions (any provider)
```

### 8.2. Shared Template Variables

All agent templates prompt for the same variables:

| Variable | Type | Description |
|---|---|---|
| `project_name` | string | Python-valid package name |
| `description` | string | One-sentence agent description |
| `trigger` | choice | `user_request` / `schedule` / `event` / `upstream_system` |
| `success_criteria` | string | What success looks like |
| `reads` | string | Data sources the agent reads |
| `writes` | string | Systems the agent writes to |
| `autonomous` | bool | Act without confirmation |
| `example_input` | string | Seeded into eval case |
| `example_output` | string | Seeded into eval case |
| `bad_output` | string | Seeded into system prompt constraints |
| `deploy_target` | choice | `job` / `serving_endpoint` |

### 8.3. Generated Structure

```
my-agent/
├── baml_src/               # (agent-baml only) BAML function definitions
├── src/my_agent/
│   ├── agent.py            # Entry point
│   ├── tools/              # Tool implementations (anthropic/openai only)
│   └── prompts/system.md   # System prompt
├── evals/                  # Eval runner + example cases
├── resources/              # DAB job and serving endpoint definitions
├── tests/
├── CLAUDE.md               # Project instructions for Claude Code
├── databricks.yml          # DAB root config
└── pyproject.toml
```

### 8.4. Framework Differences

| | `agent-anthropic` | `agent-openai` | `agent-baml` |
|---|---|---|---|
| SDK | `anthropic` | `openai` | `baml-py` |
| Prompt location | `prompts/system.md` | `prompts/system.md` | `baml_src/*.baml` |
| Output schema | untyped string | untyped string | typed via BAML class |
| Tool pattern | `tools/` autodiscovery | `tools/` autodiscovery | BAML function args |

## 9. Skills System (`dex skills`)

### 9.1. Overview

`dex skills` manages AI agent skill packs — collections of AI agent skills (slash
commands and agent personas) for tools like Claude Code, Cursor, and GitHub Copilot.

Skills are plain markdown files (source-of-truth) installed into tool-specific
locations. Packs are versioned and distributed via git repositories.

```
dex skills init                 # interactive: pick packs + targets, install
dex skills list                 # list available packs
dex skills list --verbose       # show individual skills within each pack
dex skills add <url>            # register a remote skill pack repository
dex skills sync                 # re-install per dex.toml [skills] config
dex skills sync --update        # fetch latest from remotes first
```

### 9.2. Skill Pack Format

A skill pack is a directory containing a `skills.toml` manifest and markdown files:

```
my-pack/
  skills.toml
  commands/          ← slash commands (one .md per skill)
    deploy.md
  agents/            ← agent personas (one .md per skill)
    data-engineer.md
```

Manifest format (`skills.toml`):

```toml
[pack]
name        = "my-pack"
description = "Description shown in dex skills list"
version     = "1.0.0"

[[skills]]
name        = "deploy"
type        = "command"   # "command" | "agent"
file        = "commands/deploy.md"
description = "Deploy to production"
```

### 9.3. Install Targets

Skills are installed into tool-specific directories:

| Target    | Command location                | Agent location                  |
|-----------|---------------------------------|---------------------------------|
| `claude`  | `.claude/commands/<name>.md`    | `.claude/agents/<name>.md`      |
| `cursor`  | `.cursor/rules/<name>.mdc`      | `.cursor/rules/<name>.mdc`      |
| `copilot` | `.github/copilot-instructions.md` (appended)  | same          |
| `generic` | `.ai-skills/commands/<name>.md` | `.ai-skills/agents/<name>.md`   |

### 9.4. Configuration

User config (`~/.config/dex/config.toml`):

```toml
[skills]
dir = "~/my-org-skills"          # local pack directory

[[skills.remotes]]
name = "my-org"
url  = "https://github.com/my-org/dex-skills.git"
ref  = "main"
```

Project config (`dex.toml`):

```toml
[skills]
packs   = ["default", "my-org"]
targets = ["claude", "cursor"]
```

Template integration (`template.toml`):

```toml
[skills]
packs = ["my-org"]   # suggested packs — shown as hint after dex init
```

### 9.5. Built-in Packs

dex ships two built-in skill packs embedded in the binary:

**`default`** — General-purpose development skills:
- Commands: `build`, `test`, `lint`, `review-pr`, `commit`
- Agents: `architect`, `code-reviewer`, `common-sense`

**`databricks`** — Databricks workflow skills:
- Commands: `deploy-bundle`, `run-job`
- Agents: `data-engineer`, `platform-engineer`

### 9.6. Authoring Custom Packs

See [docs/skills-authoring.md](skills-authoring.md) for the complete authoring guide.

Organizations distribute skill packs via git repos:

```bash
# Register a remote pack
dex skills add https://github.com/my-org/dex-skills.git --name my-org

# Install interactively
dex skills init

# Reproducible install via dex.toml
dex skills sync
```

## 10. v0.1 Scope

**Ship: `dex init` with built-in templates.**

### What's In

- `dex init` command with interactive prompts
- Built-in templates for Databricks projects
- Template rendering via minijinja (Jinja2 syntax)
- `template.toml` manifest format with variable declarations
- Terminal output (colors, spinners, styled prompts via console/dialoguer)
- `dex.toml` project config
- Native binary installable from GitHub Releases
- Pass-through commands via `dex.toml`

### What's Out (Future)

- `dex add`, `dex switch`, `dex deploy`
- Hooks
- `dex self update`
