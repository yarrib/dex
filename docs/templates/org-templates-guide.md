# Building Org Templates

This guide walks through building a custom template for your team — from first file to everyone on your org using it seamlessly alongside the dex built-ins.

## What you're building

A custom template works exactly like a built-in: users run `dex init --template your-name`, get interactive prompts, and a project appears. The only difference is that you authored it.

---

## 1. Pick a name and create the directory

Template names must be unique within the set of templates dex sees (built-ins + your org templates). Use a prefix to avoid collisions:

```bash
mkdir -p acme-etl/files
```

Your template directory should be inside a dedicated templates repo or within your org CLI package:

```
acme-dex-templates/
├── acme-etl/
├── acme-ml/
└── acme-serving/
```

---

## 2. Write the manifest

Every template needs a `template.toml` at its root. This defines metadata and the variables users will be prompted for.

**`acme-etl/template.toml`:**

```toml
[template]
name        = "acme-etl"
description = "Acme standard DLT ingestion pipeline"
version     = "0.1.0"
min_dex_version = "0.1.0"

# --- Variables ---

[[variables]]
name     = "project_name"
prompt   = "Project name"
type     = "string"
required = true
validate = "^[a-z][a-z0-9_]*$"     # enforce snake_case

[[variables]]
name     = "python_version"
prompt   = "Python version"
type     = "choice"
choices  = ["3.12", "3.11"]
default  = "3.12"

[[variables]]
name     = "source_path"
prompt   = "Autoloader source path"
type     = "string"
default  = "abfss://raw@acme.dfs.core.windows.net/landing/"

[[variables]]
name     = "use_serverless"
prompt   = "Use serverless compute?"
type     = "bool"
default  = false

[[variables]]
name     = "include_notebook"
prompt   = "Include exploration notebook?"
type     = "bool"
default  = true

# --- File rules (conditional inclusion) ---

[[files]]
src       = "notebooks/"
condition = "include_notebook"
```

**Variable tips:**
- Put `project_name` first — users expect it.
- Use `validate` on `project_name` to catch naming mistakes early.
- Give every optional variable a sensible `default` so `--no-prompt` works correctly.
- Use `bool` + `[[files]]` to gate optional sections rather than cluttering file content with `{% if %}` blocks.

---

## 3. Write the template files

Template files live in `files/`. Files with a `.j2` extension are rendered through minijinja (Jinja2 syntax). Files without `.j2` are copied verbatim.

```
acme-etl/
└── files/
    ├── dex.toml.j2
    ├── pyproject.toml.j2
    ├── databricks.yml.j2
    ├── README.md.j2
    ├── .gitignore
    ├── src/
    │   └── {{ project_name }}/     ← directory name uses variable
    │       ├── __init__.py
    │       └── pipeline.py.j2
    ├── resources/
    │   └── {{ project_name }}_pipeline.yml.j2
    ├── tests/
    │   ├── __init__.py
    │   └── test_{{ project_name }}.py.j2
    └── notebooks/                  ← gated by include_notebook
        └── exploration.py.j2
```

**Variable substitution in filenames and directory names:**

dex substitutes `{{ variable }}` in both file and directory names before writing. No `.j2` extension needed for the name itself — only the file *content* needs `.j2`.

**Example file — `src/{{ project_name }}/__init__.py`:**

```python
"""{{ project_name }}"""
```

(Copied verbatim — no `.j2` needed for trivial files.)

**Example file — `dex.toml.j2`:**

```toml
[project]
name     = "{{ project_name }}"
template = "acme-etl"

[passthrough.db]
command     = "databricks"
description = "Databricks CLI"

[passthrough.dlt]
command     = "databricks"
description = "DLT pipeline operations"
```

**Example file — `pyproject.toml.j2`:**

```toml
[project]
name            = "{{ project_name }}"
version         = "0.1.0"
requires-python = ">={{ python_version }}"
dependencies    = [
    "databricks-sdk>=0.20",
    "delta-spark>=3.0",
]

[dependency-groups]
dev = [
    "pytest>=8",
    "ruff>=0.5",
    "databricks-connect>=15.0",
]
```

**Example file — `resources/{{ project_name }}_pipeline.yml.j2`:**

```yaml
resources:
  pipelines:
    {{ project_name }}_pipeline:
      name: "{{ project_name }}"
      serverless: {{ use_serverless | lower }}
      libraries:
        - notebook:
            path: /Workspace/${workspace.root_path}/notebooks/{{ project_name }}
```

**Jinja2 reference for template authors:**

| Syntax | Purpose |
|---|---|
| `{{ variable }}` | Insert variable value |
| `{% if condition %}...{% endif %}` | Conditional block |
| `{% for item in list %}...{% endfor %}` | Loop |
| `{{ value \| lower }}` | Apply filter (`lower`, `upper`, `title`) |
| `{# comment #}` | Template comment (not written to output) |

---

## 4. Test locally

Before sharing, test that the template renders correctly. Point dex at your templates directory with `--template-dir` (or configure it in user config — see step 5):

```bash
# Interactive
dex init --template-dir ~/acme-dex-templates --template acme-etl --dir /tmp/test-etl

# Non-interactive (tests --no-prompt path + all defaults)
dex init --template-dir ~/acme-dex-templates --template acme-etl --no-prompt --dir /tmp/test-etl-defaults
```

Check the output:

```bash
# Verify files exist
ls /tmp/test-etl

# Verify variable substitution
cat /tmp/test-etl/dex.toml
cat /tmp/test-etl/pyproject.toml

# Verify directory names were substituted
ls /tmp/test-etl/src/
```

**Common issues:**

| Symptom | Fix |
|---|---|
| Directory not created | Check `condition` variable name matches `[[variables]]` name exactly |
| `{{ variable }}` appears literally | File is missing `.j2` extension |
| Render error | Check Jinja2 syntax — missing `%}` or unclosed block |
| `validate` rejects valid input | Test the regex: `echo "my_project" \| grep -P "^[a-z][a-z0-9_]*$"` |

---

## 5. Make it available to your team

There are two ways to share templates across your org. Use whichever fits your infrastructure.

### Option A: Shared git repository

Host the templates directory as a git repo. Team members clone it once and point dex at it.

```bash
# Team member setup (run once)
git clone https://github.com/acme/acme-dex-templates ~/acme-dex-templates
```

Add to `~/.config/dex/config.toml`:

```toml
[templates]
paths = ["~/acme-dex-templates"]
```

To update to the latest templates:

```bash
git -C ~/acme-dex-templates pull
```

**Pin a specific version** in your onboarding docs:

```bash
git clone --branch v1.2.0 https://github.com/acme/acme-dex-templates ~/acme-dex-templates
```

### Option B: Project-local templates

For templates that belong to a specific project or monorepo, commit them into a `templates/` directory at the repo root. dex discovers them automatically — no config needed.

```
my-platform/
├── dex.toml
└── templates/
    └── acme-etl/
        ├── template.toml
        └── files/
```

Anyone who clones the repo gets the template immediately.

---

## 6. Ship a standards file (optional)

A standards file pre-fills variables that are the same across all projects at your org — author names, default storage accounts, Python version policy. This removes repetitive prompts.

**`acme-standards.toml`** (checked into a shared repo or wiki):

```toml
python_version = "3.12"
source_path    = "abfss://raw@acme.dfs.core.windows.net/landing/"
```

Team members reference it with `--standards`:

```bash
dex init --template acme-etl --standards ~/acme-standards.toml --dir my_pipeline
```

Or set a default in `~/.config/dex/config.toml`:

```toml
[defaults]
standards_file = "~/acme-standards.toml"
```

---

## 7. Full team onboarding checklist

Once templates are authored and hosted, onboarding a new engineer takes two minutes:

```bash
# 1. Install dex
curl -sSf https://raw.githubusercontent.com/yarrib/dex/main/install.sh | sh

# 2. Clone templates
git clone https://github.com/acme/acme-dex-templates ~/acme-dex-templates

# 3. Configure dex
cat >> ~/.config/dex/config.toml <<'EOF'
[templates]
paths = ["~/acme-dex-templates"]
EOF

# 4. Done — scaffold a project
dex init --template acme-etl --dir my_pipeline
```

---

## See also

- [Template Authoring Guide](authoring.md) — full `template.toml` reference
- [Built-in Templates](built-in.md) — study the built-ins as worked examples
- [dex init](../usage/init.md) — all CLI flags including `--preset` and `--standards`
