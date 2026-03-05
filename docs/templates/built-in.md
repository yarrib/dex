# Built-in Templates

dex ships five built-in templates, embedded in the binary at compile time. All support `--no-prompt` for CI/scripting use.

## Choosing a template

| Template | Use when... |
|---|---|
| `default` | Starting a plain Python package or prototyping |
| `dabs-package` | Building a production Databricks job in Python |
| `dabs-etl` | Building a DLT pipeline with Autoloader ingestion |
| `dabs-ml` | Training a model with MLflow and optionally serving it |
| `dabs-aiagent` | Deploying an AI agent via mlflow.pyfunc + model serving |

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
