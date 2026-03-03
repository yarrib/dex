# PRD: dex Templates

**Status:** Planning
**Target:** v0.2
**Owner:** TBD

---

## Overview

Five first-party templates covering the core Databricks MLOps project types.
Each template is either standalone (dex renders all files) or DABs-composite
(dex delegates Phase 1 to `databricks bundle init`, then overlays Phase 2 files).

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

**DABs mode:** composite — `databricks bundle init default-python` for Phase 1,
dex overlay for Phase 2.

**Variable spec:**

| Name | Type | Default | Required | Validation |
|------|------|---------|----------|------------|
| `project_name` | string | — | yes | `^[a-z][a-z0-9_]*$` |
| `python_version` | choice | `3.12` | no | — |
| `include_dlt` | bool | `true` | no | — |
| `catalog` | string | `main` | no | — |
| `schema` | string | `default` | no | — |

**Variable map (dex → DABs):**
```toml
[template.dabs]
source = "default-python"
variable_map = { project_name = "project_name" }
```

**Generated file tree (Phase 2 overlay):**
```
<project_name>/
├── .dex/                       # dex metadata
│   └── template.lock
├── src/<project_name>/
│   └── pipeline.py             # DLT pipeline stub (if include_dlt)
└── tests/
    └── test_pipeline.py
```

**Design decisions:**
- Phase 1 (`databricks bundle init`) handles the standard DABs boilerplate.
- Phase 2 overlay adds opinionated test structure and DLT stub.
- `include_dlt` controls whether a DLT pipeline or plain job is the entry point.

---

### 4. `dabs-ml`

**Purpose:** DABs project for ML training, evaluation, and model registration workflows.

**DABs mode:** composite — `databricks bundle init mlops-stacks` for Phase 1.

**Variable spec:**

| Name | Type | Default | Required | Validation |
|------|------|---------|----------|------------|
| `project_name` | string | — | yes | `^[a-z][a-z0-9_]*$` |
| `python_version` | choice | `3.12` | no | — |
| `model_framework` | choice | `sklearn` | no | `sklearn\|pytorch\|xgboost` |
| `catalog` | string | `main` | no | — |
| `experiment_name` | string | `/<project_name>` | no | — |

**Variable map (dex → DABs mlops-stacks):**
```toml
[template.dabs]
source = "mlops-stacks"
variable_map = { project_name = "project_name", catalog = "default_catalog" }
```

**Generated file tree (Phase 2 overlay):**
```
<project_name>/
├── notebooks/
│   └── exploration.py          # scratch notebook stub
├── src/<project_name>/
│   ├── train.py                # training entry point
│   └── evaluate.py             # evaluation entry point
└── tests/
    └── test_train.py
```

**Design decisions:**
- MLflow tracking assumed (DABs mlops-stacks includes it).
- `model_framework` controls which training stub is generated.
- `experiment_name` defaults to `/<project_name>` — matches MLflow convention.

---

### 5. `dabs-aiagent`

**Purpose:** DABs project for deploying a Claude-based AI agent as a Databricks job.

**DABs mode:** composite — `databricks bundle init default-python` for Phase 1,
heavy dex overlay for Phase 2 (agent files, CLAUDE.md, system prompt).

**Variable spec:**

| Name | Type | Default | Required | Validation |
|------|------|---------|----------|------------|
| `project_name` | string | — | yes | `^[a-z][a-z0-9_]*$` |
| `agent_name` | string | — | yes | — |
| `agent_description` | string | — | no | — |
| `python_version` | choice | `3.12` | no | — |
| `generate_system_prompt` | bool | `true` | no | — |
| `catalog` | string | `main` | no | — |

**Generated file tree (Phase 2 overlay):**
```
<project_name>/
├── CLAUDE.md                   # agent-specific Claude Code instructions
├── agent.py                    # agent entry point with run()
├── system_prompt.md            # generated or placeholder system prompt
├── databricks.yml              # DABs bundle with job config
├── requirements.txt
└── tests/
    └── test_agent.py
```

**Design decisions:**
- `generate_system_prompt` triggers LLM call (same path as `dex agent new`).
- `CLAUDE.md` is agent-specific — different from the repo CLAUDE.md.
- Job config in `databricks.yml` targets a Python task (not notebook).

---

## Open Questions

1. Should composite mode be triggered by `template.toml` `[template.dabs]` presence,
   or by an explicit `--mode` flag? (Current plan: `template.toml` controls this.)
2. How do we handle DABs template version pinning? (e.g., `default-python@1.0.0`)
3. Should `dabs-aiagent` be a separate template or an overlay flag on `dabs-package`?
4. Phase 2 overlay: do we allow overwriting Phase 1 files? (Current: `FileRule.overwrite`
   exists but is not checked — bug in `scaffold.rs`.)
