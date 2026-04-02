# PRD: dex Templates

**Status:** Planning
**Target:** v0.2
**Owner:** TBD

---

## Overview

Five first-party templates covering the core Databricks MLOps project types.
All templates are standalone — dex renders all files via minijinja. No external
CLI delegation.

---

## Templates

### 1. `python-package`

**Purpose:** Bare Python package for shared utilities, models, or libraries.
No DABs bundle. No Databricks-specific runtime config.

**DABs mode:** standalone

**Variable spec:**

| Name | Type | Default | Required | Validation |
|------|------|---------|----------|------------|
| `project_name` | string | — | yes | `^[a-z][a-z0-9_]*$` |
| `python_version` | choice | `3.12` | no | `3.10\|3.11\|3.12` |
| `author` | string | git user.name | no | — |

**Generated file tree:**
```
<project_name>/
├── pyproject.toml          # hatchling build, uv config, ruff config
├── README.md
├── src/
│   └── <project_name>/
│       └── __init__.py
└── tests/
    └── __init__.py
```

**Design decisions:**
- No `databricks.yml` — this is a pure Python package.
- `src/` layout enforced (prevents import-from-repo bugs).
- hatchling build backend (simple, uv-native).

---

### 2. `dabs-package`

**Purpose:** Databricks Asset Bundle wrapping a Python package. For libraries
deployed as DABs artifacts (e.g., shared pipelines package).

**DABs mode:** standalone (dex renders all files including `databricks.yml`)

**Variable spec:**

| Name | Type | Default | Required | Validation |
|------|------|---------|----------|------------|
| `project_name` | string | — | yes | `^[a-z][a-z0-9_]*$` |
| `python_version` | choice | `3.12` | no | — |
| `databricks_host` | string | — | no | URL format |
| `catalog` | string | `main` | no | — |

**Generated file tree:**
```
<project_name>/
├── databricks.yml
├── pyproject.toml
├── README.md
├── src/
│   └── <project_name>/
│       └── __init__.py
└── tests/
    └── __init__.py
```

**Design decisions:**
- `databricks.yml` is minimal: name, bundle, targets (dev + prod stubs).
- Catalog defaulted to `main`; users override in `databricks.yml`.

---

### 3. `dabs-etl`

**Purpose:** DABs project with a DLT pipeline or job for ETL workloads.

**Rendering mode:** standalone

**Variable spec:**

| Name | Type | Default | Required | Validation |
|------|------|---------|----------|------------|
| `project_name` | string | — | yes | `^[a-z][a-z0-9_]*$` |
| `python_version` | choice | `3.12` | no | — |
| `include_dlt` | bool | `true` | no | — |
| `catalog` | string | `main` | no | — |
| `schema` | string | `default` | no | — |

**Generated file tree:**
```
<project_name>/
├── databricks.yml
├── pyproject.toml
├── README.md
├── .gitignore
├── src/<project_name>/
│   ├── __init__.py
│   └── pipeline.py             # DLT pipeline stub (if include_dlt)
├── resources/
│   └── pipeline_job.yml        # job definition
└── tests/
    └── test_pipeline.py
```

**Design decisions:**
- dex renders all files including `databricks.yml` — no dependency on the Databricks CLI at scaffold time.
- `include_dlt` controls whether a DLT pipeline or plain job entry point is generated.

---

### 4. `dabs-ml`

**Purpose:** DABs project for ML training, evaluation, and model registration workflows.

**Rendering mode:** standalone

**Variable spec:**

| Name | Type | Default | Required | Validation |
|------|------|---------|----------|------------|
| `project_name` | string | — | yes | `^[a-z][a-z0-9_]*$` |
| `python_version` | choice | `3.12` | no | — |
| `model_framework` | choice | `sklearn` | no | `sklearn\|pytorch\|xgboost` |
| `catalog` | string | `main` | no | — |
| `experiment_name` | string | `/<project_name>` | no | — |
| `include_serving` | bool | `true` | no | — |

**Generated file tree:**
```
<project_name>/
├── databricks.yml
├── pyproject.toml
├── README.md
├── .gitignore
├── notebooks/
│   └── exploration.py          # scratch notebook stub
├── src/<project_name>/
│   ├── __init__.py
│   ├── train.py                # training entry point
│   └── evaluate.py             # evaluation entry point
├── resources/
│   └── ml_job.yml
├── serving/                    # present if include_serving = true
│   └── endpoint.yml
└── tests/
    └── test_train.py
```

**Design decisions:**
- dex renders all files including `databricks.yml` and MLflow config — no dependency on `mlops-stacks`.
- `model_framework` controls which training stub and dependency is generated.
- `experiment_name` defaults to `/<project_name>` — matches MLflow convention.

---

### 5. `dabs-aiagent`

**Purpose:** DABs project for deploying a Claude-based AI agent as a Databricks job.

**Rendering mode:** standalone

**Variable spec:**

| Name | Type | Default | Required | Validation |
|------|------|---------|----------|------------|
| `project_name` | string | — | yes | `^[a-z][a-z0-9_]*$` |
| `agent_name` | string | — | yes | — |
| `agent_description` | string | — | no | — |
| `python_version` | choice | `3.12` | no | — |
| `generate_system_prompt` | bool | `true` | no | — |
| `catalog` | string | `main` | no | — |

**Generated file tree:**
```
<project_name>/
├── databricks.yml              # DABs bundle with job config
├── pyproject.toml
├── README.md
├── .gitignore
├── CLAUDE.md                   # agent-specific Claude Code instructions
├── src/<project_name>/
│   ├── __init__.py
│   ├── agent.py                # agent entry point with run()
│   └── tools/                  # present if include_vector_search = true
│       └── retriever.py
├── resources/
│   └── agent_job.yml
├── system_prompt.md            # generated or placeholder system prompt
└── tests/
    └── test_agent.py
```

**Design decisions:**
- dex renders all files including `databricks.yml` — no dependency on the Databricks CLI at scaffold time.
- `generate_system_prompt` triggers LLM call (same path as `dex agent new`).
- `CLAUDE.md` is agent-specific — different from the repo CLAUDE.md.
- Job config in `databricks.yml` targets a Python task (not notebook).

---

## Open Questions

1. Should `dabs-aiagent` be a separate template or an overlay flag on `dabs-package`?
2. `FileRule.overwrite` exists and is checked in `scaffold.rs` — should re-init flows
   (e.g. `dex init --force`) use this to selectively overwrite config files while
   preserving source files?
