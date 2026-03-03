# Template Reference

## default

A general-purpose Python + Databricks project.

**Variables:**

| Name | Type | Default | Description |
|---|---|---|---|
| `project_name` | string | *(dir name)* | Python package name (lowercase, underscores) |
| `python_version` | choice | `3.12` | Python version (`3.12`, `3.11`) |

**Generated files:**

```
src/<project_name>/
├── __init__.py
└── main.py
notebooks/
└── exploration.py
tests/
├── __init__.py
└── test_main.py
databricks.yml
pyproject.toml
README.md
.gitignore
```

---

## dabs-package

A full Databricks Asset Bundle Python package. Includes job definitions, multi-target deploy config, and optional notebook scaffolding.

**Variables:**

| Name | Type | Default | Description |
|---|---|---|---|
| `project_name` | string | *(dir name)* | Python package name (lowercase, underscores) |
| `python_version` | choice | `3.12` | Python version (`3.12`, `3.11`) |
| `include_notebook` | bool | `true` | Generate `notebooks/exploration.py` |
| `include_job` | bool | `true` | Generate `resources/<project_name>_job.yml` |
| `use_serverless` | bool | `false` | Use serverless compute in the job definition |

**Generated files:**

```
src/<project_name>/
├── __init__.py
└── main.py              # entry point with --catalog / --schema args
resources/               # only if include_job=true
└── <project_name>_job.yml
notebooks/               # only if include_notebook=true
└── exploration.py
tests/
├── __init__.py
└── test_<project_name>.py
databricks.yml           # bundle config: artifacts, targets, variables
pyproject.toml           # dev deps: pytest, ruff, databricks-connect
README.md
.gitignore
```

**DABs targets:**

The generated `databricks.yml` includes three targets:

- `dev` (default) — `mode: development`, catalog `dev`
- `staging` — `mode: development`, catalog `staging`
- `prod` — production catalog

Deploy with:

```bash
databricks bundle deploy              # → dev
databricks bundle deploy --target prod
```
