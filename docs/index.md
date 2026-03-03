# dex

**dex** is an opinionated CLI framework for data project operations. It scaffolds Python packages, Databricks Asset Bundles, and AI agent projects — and can be extended by teams to wrap their own tooling.

## Quick install

```bash
pip install dex
```

Or with uv:

```bash
uv add dex
```

## 30-second example

```bash
# Scaffold a new Databricks Asset Bundle project
dex init --template dabs-package my_project

# Scaffold a plain Python package
dex init --template default my_package

# Non-interactive (use all defaults)
dex init --template dabs-package --no-prompt my_project
```

## What dex generates

For a `dabs-package` project:

```
my_project/
├── src/my_project/
│   ├── __init__.py
│   └── main.py          # entry point with argparse
├── resources/
│   └── my_project_job.yml   # DABs job definition
├── notebooks/
│   └── exploration.py   # Databricks notebook
├── tests/
│   ├── __init__.py
│   └── test_my_project.py
├── databricks.yml       # bundle config (dev/staging/prod targets)
├── pyproject.toml       # uv-compatible project config
├── README.md
└── .gitignore
```

## Next steps

- [Installation](installation.md) — pip, uv, build from source
- [Usage: dex init](usage/init.md) — all options and templates
- [Templates](usage/templates.md) — template reference
