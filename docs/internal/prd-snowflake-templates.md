# PRD: Snowflake Templates

**Status:** Planning
**Target:** v0.3
**Owner:** TBD

---

## Overview

Four first-party templates covering the core Snowflake project types: pure Python
packages for Snowflake utilities, Snowpark ETL pipelines, Snowpark ML projects,
and dbt-based transformation projects. All four are standalone mode — dex renders
all files directly, with no external `bundle init` step.

Snowflake projects differ from Databricks projects in key ways:
- Connection config (account, warehouse, database, schema, role) replaces cluster/workspace config.
- Snowpark is the primary Python data processing API (replaces Spark).
- Tasks and streams replace DABs job scheduling.
- No DABs composite mode — all templates are rendered entirely by dex.

---

## Templates

### 1. `snowflake-python`

**Purpose:** Pure Python package for Snowflake utilities: UDFs, stored procedures,
shared helpers, or Snowpark transformations intended for reuse across projects.
No Snowflake tasks or deployment config — just a tested, installable Python package.

**Rendering mode:** standalone

**Variable spec:**

| Name | Type | Default | Required | Validation |
|------|------|---------|----------|------------|
| `project_name` | string | — | yes | `^[a-z][a-z0-9_]*$` |
| `python_version` | choice | `3.11` | no | `3.10\|3.11\|3.12` |
| `author` | string | `""` | no | — |
| `snowflake_account` | string | `""` | no | — |

**Generated file tree:**
```
<project_name>/
├── pyproject.toml              # hatchling build, uv config, ruff config
├── README.md
├── .gitignore
├── src/
│   └── <project_name>/
│       ├── __init__.py
│       └── utils.py            # placeholder Snowpark helper
└── tests/
    ├── __init__.py
    └── test_utils.py
```

**Design decisions:**
- No `snowflake.yml` or connection file — connection config is injected at runtime
  via environment variables (`SNOWFLAKE_ACCOUNT`, `SNOWFLAKE_USER`, etc.) or
  `~/.snowflake/config.toml`.
- `src/` layout enforced (same as `python-package`).
- `snowflake-snowpark-python` added to dev deps in `pyproject.toml`; not pinned to
  a specific version — users manage Snowflake SDK pinning themselves.
- `author` and `snowflake_account` are optional — useful for teams generating
  boilerplate but not required for a working package.

---

### 2. `snowflake-etl`

**Purpose:** Snowpark ETL project with a pipeline entry point, Snowflake task
definition, and optional stream-based CDC trigger. For teams running scheduled
data ingestion, transformation, or loading jobs inside Snowflake.

**Rendering mode:** standalone

**Variable spec:**

| Name | Type | Default | Required | Validation |
|------|------|---------|----------|------------|
| `project_name` | string | — | yes | `^[a-z][a-z0-9_]*$` |
| `python_version` | choice | `3.11` | no | `3.10\|3.11\|3.12` |
| `snowflake_account` | string | `""` | no | — |
| `database` | string | `""` | no | — |
| `schema` | string | `PUBLIC` | no | — |
| `warehouse` | string | `COMPUTE_WH` | no | — |
| `include_stream` | bool | `false` | no | — |
| `include_task` | bool | `true` | no | — |

**Generated file tree:**
```
<project_name>/
├── pyproject.toml
├── README.md
├── .gitignore
├── snowflake.yml               # connection profile (gitignored values)
├── deploy.sql                  # CREATE TASK / CREATE STREAM DDL stubs
├── src/
│   └── <project_name>/
│       ├── __init__.py
│       ├── pipeline.py         # Snowpark DataFrame transformation entry point
│       └── loader.py           # target table write helpers
├── streams/                    # present if include_stream = true
│   └── source_stream.sql       # CREATE STREAM stub
└── tests/
    ├── __init__.py
    └── test_pipeline.py
```

**Conditional files:**

| Condition | Files |
|-----------|-------|
| `include_stream` | `streams/source_stream.sql` |
| `include_task` | `deploy.sql` includes CREATE TASK DDL |

**Design decisions:**
- `snowflake.yml` is generated but listed in `.gitignore` — it holds connection
  profile values for local dev. Production credentials come from environment
  variables or Secrets Manager.
- `pipeline.py` uses `snowflake.snowpark.Session` directly, not a framework
  abstraction. Keeps the entry point readable and portable.
- `deploy.sql` contains the full DDL to create the Snowflake task pointing at
  the Snowpark stored procedure. Intended for `snowsql -f deploy.sql` or
  Terraform-managed deployment.
- Stream support is optional because not all ETL patterns need CDC — batch
  ingestion from external stages is common and doesn't require streams.

---

### 3. `snowflake-ml`

**Purpose:** Snowpark ML project for feature engineering, model training with
Snowflake Model Registry, and optional Cortex LLM function usage. For teams
doing ML entirely within Snowflake's compute boundary.

**Rendering mode:** standalone

**Variable spec:**

| Name | Type | Default | Required | Validation |
|------|------|---------|----------|------------|
| `project_name` | string | — | yes | `^[a-z][a-z0-9_]*$` |
| `python_version` | choice | `3.11` | no | `3.10\|3.11\|3.12` |
| `snowflake_account` | string | `""` | no | — |
| `database` | string | `""` | no | — |
| `schema` | string | `PUBLIC` | no | — |
| `warehouse` | string | `COMPUTE_WH` | no | — |
| `model_framework` | choice | `sklearn` | no | `sklearn\|xgboost\|lightgbm` |
| `include_cortex` | bool | `false` | no | — |
| `include_feature_store` | bool | `false` | no | — |

**Generated file tree:**
```
<project_name>/
├── pyproject.toml
├── README.md
├── .gitignore
├── snowflake.yml
├── src/
│   └── <project_name>/
│       ├── __init__.py
│       ├── features.py         # Snowpark feature engineering
│       ├── train.py            # model training + registry registration
│       ├── evaluate.py         # evaluation against holdout
│       └── cortex.py           # Cortex LLM helpers (if include_cortex)
├── feature_store/              # present if include_feature_store = true
│   └── feature_views.sql       # CREATE FEATURE VIEW stubs
├── notebooks/
│   └── exploration.ipynb       # Snowflake Notebooks-compatible stub
└── tests/
    ├── __init__.py
    ├── test_features.py
    └── test_train.py
```

**Conditional files:**

| Condition | Files |
|-----------|-------|
| `include_cortex` | `src/<project_name>/cortex.py` |
| `include_feature_store` | `feature_store/feature_views.sql` |

**Design decisions:**
- `model_framework` controls which training stub and dependency is generated
  (`scikit-learn`, `xgboost`, or `lightgbm`). All three are supported by
  Snowflake Model Registry's `snowflake.ml.modeling` wrappers.
- Model registration targets Snowflake Model Registry (not MLflow). Teams already
  on Databricks with MLflow tracking should use `dabs-ml` instead.
- `include_cortex` adds `cortex.py` with `snowflake.cortex.Complete()` and
  `cortex.ExtractAnswer()` helpers — useful for LLM-augmented feature enrichment.
- `include_feature_store` generates DDL stubs for Snowflake Feature Store. Kept
  optional because it requires a specific Snowflake tier and most early-stage
  projects don't need it.
- `notebooks/exploration.ipynb` is a Snowflake Notebooks-compatible stub
  (importable via Snowsight UI). Not a Jupyter notebook run locally.

---

### 4. `snowflake-dbt`

**Purpose:** dbt project targeting Snowflake as the warehouse. For teams using
dbt Core for SQL-based transformations with Snowflake as the compute and storage
layer. Includes profiles.yml stub, standard dbt project layout, and optional
dbt tests scaffolding.

**Rendering mode:** standalone

**Variable spec:**

| Name | Type | Default | Required | Validation |
|------|------|---------|----------|------------|
| `project_name` | string | — | yes | `^[a-z][a-z0-9_-]*$` |
| `python_version` | choice | `3.11` | no | `3.10\|3.11\|3.12` |
| `snowflake_account` | string | `""` | no | — |
| `database` | string | `""` | no | — |
| `schema` | string | `PUBLIC` | no | — |
| `warehouse` | string | `COMPUTE_WH` | no | — |
| `dbt_version` | choice | `1.8` | no | `1.7\|1.8\|1.9` |
| `include_sources` | bool | `true` | no | — |
| `include_tests` | bool | `true` | no | — |

**Generated file tree:**
```
<project_name>/
├── dbt_project.yml             # dbt project config (name, profile, paths)
├── profiles.yml                # Snowflake connection profile (gitignored)
├── pyproject.toml              # dbt-core + dbt-snowflake pinned to dbt_version
├── README.md
├── .gitignore
├── models/
│   ├── staging/
│   │   └── _sources.yml        # source definitions (if include_sources)
│   └── marts/
│       └── example_mart.sql    # sample SELECT model
├── tests/                      # present if include_tests = true
│   └── assert_example.sql
├── macros/
│   └── .gitkeep
└── seeds/
    └── .gitkeep
```

**Conditional files:**

| Condition | Files |
|-----------|-------|
| `include_sources` | `models/staging/_sources.yml` |
| `include_tests` | `tests/assert_example.sql` |

**Design decisions:**
- `profiles.yml` is generated and gitignored. Snowflake credentials are filled
  in locally or via `DBT_SNOWFLAKE_ACCOUNT` / `DBT_SNOWFLAKE_PASSWORD` env vars.
  dbt Cloud users don't use this file at all — the generated profile is for
  dbt Core local dev only.
- `dbt_version` controls pinning in `pyproject.toml`. dbt's minor versions
  frequently introduce breaking changes, so version pinning is intentional.
- `include_sources` generates a `_sources.yml` with a placeholder source block.
  Most real dbt projects need this; turning it off keeps the scaffold minimal
  for teams that manage sources elsewhere (e.g., a shared dbt package).
- Template uses `^[a-z][a-z0-9_-]*$` validation (allows hyphens) because dbt
  project names conventionally use hyphens (e.g., `analytics-dbt`), unlike
  Python package names which forbid them.
- No `dbt init` delegation. dex renders all dbt files directly so the scaffold
  is deterministic and auditable without network access.

---

## Connection Config Conventions

All four Snowflake templates share the same connection variable names
(`snowflake_account`, `database`, `schema`, `warehouse`) and the same pattern:

- Variables are optional in `template.toml` — a team without a standard account
  can scaffold and fill them in later.
- Generated files use `{{ snowflake_account | default("") }}` in connection stubs
  so empty values produce valid (if incomplete) config files rather than broken ones.
- `snowflake.yml` and `profiles.yml` are always listed in `.gitignore`.
  Secrets never go into source control.

---

## Template Naming Conventions

| Template | Platform | Primary tool |
|----------|----------|-------------|
| `snowflake-python` | Snowflake | Snowpark |
| `snowflake-etl` | Snowflake | Snowpark + Tasks |
| `snowflake-ml` | Snowflake | Snowpark ML + Registry |
| `snowflake-dbt` | Snowflake | dbt Core |

All Snowflake templates are prefixed `snowflake-` (not `snow-`) for clarity in
`dex init --list` output and to leave `snow-` free for potential SnowSQL-oriented
templates in future versions.

---

## Open Questions

1. **Snowpark vs. raw connector:** Should `snowflake-etl` and `snowflake-ml` use
   `snowflake-snowpark-python` (higher-level DataFrame API) or `snowflake-connector-python`
   (lower-level, more portable)? Current plan: Snowpark for both. Teams that want
   raw connector can use `snowflake-python`.

2. **dbt Cloud support:** `snowflake-dbt` is scoped to dbt Core. Should there be a
   `snowflake-dbt-cloud` variant (or a `use_dbt_cloud` bool) that omits `profiles.yml`
   and adds a `.dbt_cloud.yml` stub? Deferred to v0.4.

3. **Snowflake CLI (`snow`) passthrough:** Should `dex.toml` include a passthrough
   for `snow` (Snowflake CLI) when a Snowflake template is scaffolded? This would
   let `dex snow app run` delegate to `snow app run`. Out of scope for this PRD —
   tracked separately as a passthrough config feature.

4. **Python version floor:** Snowpark 1.x requires Python 3.8+, but Snowflake
   recommends 3.10+ for new projects. Current choice list (`3.10|3.11|3.12`) may
   need updating as Snowflake drops support for older runtimes. Templates should
   track [Snowflake Python runtime support](https://docs.snowflake.com/en/developer-guide/snowpark/python/index).

5. **Shared connection variables:** Should connection vars (`database`, `schema`,
   `warehouse`) live in a separate `[defaults.snowflake]` block in `dex.toml`
   (like a standards file) rather than being prompted per-template? This is a
   general dex config question, not specific to Snowflake, but Snowflake is the
   first platform where it matters across multiple templates.
