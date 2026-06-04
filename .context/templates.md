---
id: templates
title: templates
kind: artifact
summary: The built-in project templates embedded at compile time and rendered into new projects.
related:
  - template-engine: rendered-by
  - template-manifest: described-by
  - traits: composed-with
---

The top-level `templates/` directory holds the built-in project templates,
embedded into the binary at compile time via `include_dir`. Each template is a
directory of `.j2` files plus a `template.toml` manifest.

Built-in templates include `default`, `python-package`, the Databricks Asset
Bundle family (`dabs-package`, `dabs-etl`, `dabs-ml`, `dabs-dashboard`,
`dabs-genie-space`, `dabs-aiagent`), Databricks apps
(`databricks-app-react`, `databricks-app-streamlit`), and AI agent starters
(`agent-anthropic`, `agent-openai`, `agent-baml`).

Orgs can also supply their own via directory or remote git registries — no code
changes needed.
