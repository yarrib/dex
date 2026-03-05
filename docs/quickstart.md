# Quickstart

Install dex and scaffold your first project in under a minute.

## 1. Install

```bash
curl -sSf https://raw.githubusercontent.com/yarrib/dex/main/install.sh | sh
```

Or manually with uv (see [Installation](installation.md) for platform-specific wheel URLs).

See [Installation](installation.md) for platform-specific wheels and Windows instructions.

## 2. Scaffold a project

=== "Databricks Asset Bundle"

    ```bash
    dex init --template dabs-package --dir my_project
    ```

    Prompts:

    ```
    Project name [my_project]:
    Python version (3.12, 3.11) [3.12]:
    Include exploration notebook? [Y/n]:
    Include job definition? [Y/n]:
    Use serverless compute? [y/N]:
    ```

=== "Plain Python package"

    ```bash
    dex init --template default --dir my_package
    ```

=== "Non-interactive (CI / scripts)"

    ```bash
    dex init --template dabs-package --no-prompt --dir my_project
    ```

## 3. Inspect what was generated

```
my_project/
├── src/my_project/
│   ├── __init__.py
│   └── main.py
├── resources/
│   └── my_project_job.yml
├── notebooks/
│   └── exploration.py
├── tests/
│   └── test_my_project.py
├── databricks.yml
├── pyproject.toml
├── README.md
└── .gitignore
```

## 4. Deploy to Databricks

```bash
cd my_project
databricks bundle deploy          # → dev target
databricks bundle deploy --target prod
```

## Next steps

- [dex init reference](usage/init.md) — all options and template variables
- [Template reference](usage/templates.md) — what each template generates
- [dex agent new](usage/agent.md) — scaffold an AI agent project
