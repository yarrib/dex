# Built-in Templates

dex ships a family of built-in templates, embedded in the binary at compile time. All support `--no-prompt` for CI/scripting use.

## Choosing a template

| Template | Use when... |
|---|---|
| `default` | Starting a plain Python package or prototyping |
| **`dabs-platform`** | **Any Databricks Asset Bundle project — one profile-driven template covering data engineering, MLOps, and agent workloads (use with `--preset`)** |
| `dabs-package` | *(Deprecated — use `dabs-platform`)* Databricks job in Python |
| `dabs-etl` | *(Deprecated — use `dabs-platform` with `--preset data_eng`)* DLT pipeline |
| `dabs-ml` | *(Deprecated — use `dabs-platform` with `--preset mlops`)* MLflow training + serving |
| `dabs-aiagent` | *(Deprecated — use `dabs-platform` with `--preset agents`)* AI agent serving |
| **`agent-dbos`** | An agent whose runs must survive crashes, redeploys, or a wait for human approval (durable execution on DBOS) |

The four focused `dabs-*` templates below are **superseded by `dabs-platform`**, which
consolidates their baseline into a single template to avoid drift. They remain available
for backwards compatibility.

---

## dabs-platform

A single Databricks Asset Bundle template, driven by **profiles = presets**. One baseline
(uv, ruff, ty, pytest, dev/staging/prod targets, a base job, CI/CD callers) plus optional
components gated by boolean variables. Pick a profile with `--preset`; interactive prompting
is the power-user path.

```bash
dex init -t dabs-platform --preset data_eng \
    --presets-file templates/dabs-platform/presets.toml --dir my_bundle
```

**Profiles** (from `presets.toml`):

| Profile | Components enabled |
|---|---|
| `data_eng` | pipeline, observability (data quality) |
| `mlops` | experiment + model, realtime serving, observability (inference) |
| `agents` | experiment + model, realtime serving, observability, agent wrapper + eval |
| `extraction_agents` | agents + batch inference, `shared_code_source = workspace` (monorepo) |

**Gating variables** (all default `false`; presets turn on only what they need):
`include_pipeline`, `include_experiment`, `include_realtime`, `include_batch`,
`include_observability`, `include_agent`, `include_app`, `include_lakebase`,
`include_react`, plus `shared_code_source` (`none`/`workspace`/`path`/`wheel`).

**Generated files:** always emits `pyproject.toml`, `databricks.yml` (dev/staging/prod),
`dex.toml`, a base job, `src/`, `tests/`, and `.github/workflows/{ci,cd}.yml` that call
your org's central reusable workflows (`central_ci_ref` / `central_cd_ref`). Each enabled
component adds its own gated directory (`pipeline/`, `ml/`, `serving_rt/`, `serving_batch/`,
`observability/`, `agent/`, `app/`, `lakebase/`, `frontend/`).

> **Distribution:** point an org registry at this template with `[[templates.remotes]]`
> in `~/.config/dex/config.toml`, and ship org constants via `standards.toml`. See the
> template's own README (`templates/dabs-platform/README.md`).

---

## agent-dbos

A Python agent service whose runs are **durable**: every LLM call and tool call is a
checkpointed [DBOS](https://docs.dbos.dev/python/) step and the agent loop is a DBOS workflow.
`kill -9` mid-run, restart, and the run finishes with exactly one model call per step and one
side effect per action. It sits beside `agent-openai` / `agent-anthropic` / `agent-baml`, which
scaffold the agent logic but not its orchestration.

```bash
dex init --template agent-dbos --dir my_agent
dex agent new --runtime durable --sdk anthropic      # same template, llm_sdk preset
dex init -t agent-dbos --no-prompt --answers answers.toml
```

**When to use:** runs longer than one request — multi-step tool use, background jobs, anything
that waits on a human. For a single-shot agent, use `agent-openai` / `agent-anthropic`.

**Variables:**

| Name | Type | Default | Description |
|---|---|---|---|
| `project_name` | string | *(dir name)* | Package and app name (`^[a-z][a-z0-9_]*$`) |
| `description` | string | — | One-line purpose (README, AGENTS.md, system prompt) |
| `llm_sdk` | choice | `openai` | Client used inside the model step (`openai`, `anthropic`) |
| `database` | choice | `postgres` | DBOS system database (`postgres` → compose file; `lakebase` → OAuth-token engine) |
| `include_hitl` | bool | `true` | Approval step (`DBOS.recv`), inbox endpoints, `approve`/`reject` commands |
| `include_api` | bool | `true` | FastAPI service with SSE; `false` gives a CLI-only queue worker |
| `deploy_target` | choice | `databricks_app` | `databricks_app` adds `databricks.yml`, `resources/app.yml`, `app.yaml`; `none` omits them |
| `max_concurrent_runs` | string (int) | `4` | Queue concurrency |
| `python_version` | choice | `3.12` | Python version (`3.12`, `3.11`) |
| `ai_tools`, `include_mcp` | multi / bool | as `agent-openai` | AI assistant configuration |

Only `database`, `llm_sdk` and `deploy_target` change generated code; the other options toggle
whole files.

**Generated files (default options):**

```
src/<project_name>/
├── cli.py                  # serve | run | approve | reject | status | replay
├── config.py               # settings from env; DBOS config
├── runtime.py              # launch/shutdown, queue registration, client factory
├── workflows/agent_run.py  # @DBOS.workflow — the agent loop
├── workflows/queue.py      # "agent_runs": concurrency cap + start-rate limiter
├── steps/llm.py            # @DBOS.step model call; tokens stream live, completion checkpointed
├── steps/tools.py          # @DBOS.step per tool call, idempotency keys
├── steps/approval.py       # durable human-in-the-loop wait   (include_hitl)
├── agent/{prompts/system.md, schemas.py, tools/}
├── streaming.py            # resumable progress events (DBOS streams) + in-memory token bus
├── api.py, static/         # FastAPI: runs, SSE, approvals, inbox page   (include_api)
└── tracing.py              # MLflow spans tagged with the run id
tests/                      # fake LLM; crash tests kill a real worker process
evals/                      # runner.py + cases/*.json
docker-compose.yml          # local Postgres 16                    (database = postgres)
databricks.yml, app.yaml, resources/app.yml                        (deploy_target = databricks_app)
dbos-config.yaml, dex.toml, AGENTS.md, CLAUDE.md, README.md, .env.example
.claude/skills/durable-workflows/SKILL.md   # replay rules for coding assistants (when `claude` in ai_tools)
```

**Replay-safety rules** (stated in the generated `AGENTS.md` and skill): workflow code is
deterministic between steps; every non-deterministic call is a step; steps return
JSON-serializable values; steps are at-least-once so side effects are idempotent by key;
changing step order is a breaking change (bump `APP_VERSION`).

**Generated-project tests** run on a throwaway SQLite system database with a fake model — no
Docker, no API keys. Set `TEST_DATABASE_URL` to run the same suite on Postgres. The resume and
idempotency tests crash a real worker process with `os._exit` and recover the run in a fresh
one.

> **Pinning:** `pyproject.toml` pins `dbos~=3.4`. The queue API (`DBOS.register_queue`, called
> after launch) and workflow streams (`write_stream`/`read_stream`) are 3.x APIs.

---

## default

A minimal Python package for general-purpose development.

**When to use:** Prototyping, standalone scripts, or non-Databricks Python projects. No DABs config included.

```bash
dex init --template default --dir my_package
```

**Variables:**

| Name | Type | Default | Description |
|---|---|---|---|
| `project_name` | string | *(dir name)* | Package name (lowercase, hyphens or underscores) |
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

> **Deprecated — superseded by `dabs-platform`.** Retained for backwards compatibility.

A full Databricks Asset Bundle Python package. The go-to template for production Databricks jobs.

**When to use:** Any Databricks job that runs Python code — ETL scripts, batch jobs, data quality checks. Includes multi-target deploy config (dev/staging/prod) out of the box.

```bash
dex init --template dabs-package --dir my_project
```

**Variables:**

| Name | Type | Default | Description |
|---|---|---|---|
| `project_name` | string | *(dir name)* | Package name (lowercase, underscores) |
| `python_version` | choice | `3.12` | Python version (`3.12`, `3.11`) |
| `include_notebook` | bool | `true` | Generate `notebooks/exploration.py` |
| `include_job` | bool | `true` | Generate `resources/<name>_job.yml` |
| `use_serverless` | bool | `false` | Use serverless compute in the job definition |

**Generated files:**

```
src/<project_name>/
├── __init__.py
└── main.py                         # entry point with --catalog / --schema args
resources/                          # if include_job=true
└── <project_name>_job.yml
notebooks/                          # if include_notebook=true
└── exploration.py
tests/
├── __init__.py
└── test_<project_name>.py
databricks.yml                      # dev / staging / prod targets
pyproject.toml
README.md
.gitignore
```

---

## dabs-etl

> **Deprecated — superseded by `dabs-platform` (`--preset data_eng`).** Retained for backwards compatibility.

A DLT (Delta Live Tables) pipeline project with Autoloader ingestion.

**When to use:** Streaming or batch ingestion pipelines where data arrives in cloud storage and needs to be loaded into Delta tables. Autoloader handles schema inference and file tracking automatically.

```bash
dex init --template dabs-etl --dir my_pipeline
```

**Variables:**

| Name | Type | Default | Description |
|---|---|---|---|
| `project_name` | string | *(dir name)* | Package name (lowercase, underscores) |
| `python_version` | choice | `3.12` | Python version (`3.12`, `3.11`) |
| `source_path` | string | `abfss://raw@<storage-account>.dfs.core.windows.net/landing/` | Autoloader source path |
| `use_serverless` | bool | `false` | Use serverless compute |
| `include_notebook` | bool | `true` | Generate `notebooks/exploration.py` |

**Generated files:**

```
src/<project_name>/
├── __init__.py
└── pipeline.py                     # DLT pipeline definition
resources/
└── <project_name>_pipeline.yml     # DLT pipeline DABs resource
notebooks/                          # if include_notebook=true
└── exploration.py
tests/
├── __init__.py
└── test_<project_name>.py
databricks.yml
pyproject.toml
README.md
.gitignore
```

---

## dabs-ml

> **Deprecated — superseded by `dabs-platform` (`--preset mlops`).** Retained for backwards compatibility.

An MLflow training project with model registry integration and optional model serving.

**When to use:** Supervised ML workflows — feature engineering, training, evaluation, and registration to Unity Catalog model registry. Optionally deploys a real-time serving endpoint.

```bash
dex init --template dabs-ml --dir my_model
```

**Variables:**

| Name | Type | Default | Description |
|---|---|---|---|
| `project_name` | string | *(dir name)* | Package name (lowercase, underscores) |
| `python_version` | choice | `3.12` | Python version (`3.12`, `3.11`) |
| `use_serverless` | bool | `false` | Use serverless compute |
| `include_serving` | bool | `true` | Generate model serving endpoint config |
| `include_notebook` | bool | `true` | Generate `notebooks/exploration.py` |

**Generated files:**

```
src/<project_name>/
├── __init__.py
└── train.py                        # MLflow training script
resources/
└── <project_name>_training_job.yml
serving/                            # if include_serving=true
└── <project_name>_serving.yml      # Model serving endpoint definition
notebooks/                          # if include_notebook=true
└── exploration.py
tests/
├── __init__.py
└── test_<project_name>.py
databricks.yml
pyproject.toml
README.md
.gitignore
```

---

## dabs-aiagent

> **Deprecated — superseded by `dabs-platform` (`--preset agents`).** Retained for backwards compatibility.

An AI agent project using `mlflow.pyfunc` for packaging and Databricks model serving for deployment.

**When to use:** Building a custom AI agent or LLM-powered application that needs to be deployed as a Databricks model serving endpoint. Optionally includes a vector search retriever for RAG patterns.

```bash
dex init --template dabs-aiagent --dir my_agent
```

**Variables:**

| Name | Type | Default | Description |
|---|---|---|---|
| `project_name` | string | *(dir name)* | Package name (lowercase, underscores) |
| `python_version` | choice | `3.12` | Python version (`3.12`, `3.11`) |
| `include_vector_search` | bool | `false` | Include a vector search retriever (RAG) |
| `use_serverless` | bool | `true` | Use serverless compute for the deploy job |

**Generated files:**

```
src/<project_name>/
├── __init__.py
├── agent.py                        # mlflow.pyfunc agent wrapper
└── tools/
    ├── __init__.py
    └── retriever.py                # if include_vector_search=true
resources/
├── <project_name>_deploy_job.yml   # DABs job to register + deploy the agent
└── <project_name>_serving.yml      # Model serving endpoint definition
notebooks/
├── deploy_agent.py                 # notebook: register model + deploy endpoint
└── evaluate_agent.py               # notebook: evaluate agent with MLflow
tests/
├── __init__.py
└── test_<project_name>.py
databricks.yml
pyproject.toml
README.md
.gitignore
```
