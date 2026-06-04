# dex

**dex** is an opinionated CLI framework for data project operations. It scaffolds Python packages, Databricks Asset Bundles, and AI agent projects — and can be extended by teams to wrap their own tooling.

100% Rust. Single binary. No runtime required.

## Install

```bash
curl -sSf https://raw.githubusercontent.com/yarrib/dex/main/install.sh | sh
```

Auto-detects your platform and downloads the right binary from [GitHub Releases](https://github.com/yarrib/dex/releases).
See [Installation](installation.md) for manual install, Windows, and build-from-source options.

## 30-second example

```bash
# Scaffold a new Databricks Asset Bundle project
dex init --template dabs-package --dir my_project

# Scaffold a plain Python package
dex init --template default --dir my_package

# Non-interactive (use all defaults)
dex init --template dabs-package --no-prompt --dir my_project
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
├── pyproject.toml       # project config
├── dex.toml             # dex project config
├── README.md
└── .gitignore
```

## Next steps

- [Installation](installation.md) — install from GitHub Releases or build from source
- [Usage: dex init](usage/init.md) — all options and templates
- [Templates](usage/templates.md) — template reference
- [Set up dex for your org](../examples/README.md) — runnable example: org templates, config, standards, presets
- [Building Org Templates](templates/org-templates-guide.md) — author and share templates with your team
