---
sha: 0752c04b9dc8546ea307d6a07fb50a76a185b73f
short_sha: 0752c04
author: yarrib
date: 2026-04-02
class: [Evolution]
area: CLI & Interfaces
tags: [#evolution]
---

# [Evolution] feat(cli): add dex templates list/show (#40)

**Commit:** `0752c04` · **Author:** yarrib · **Date:** 2026-04-02 · **Area:** CLI & Interfaces

## Summary

- `dex templates list` — lists all built-in (and remote) templates with
name + description
- `dex templates list --verbose` — also shows variables inline for each
template
- `dex templates show <name>` — full detail view: description, version,
variables with defaults, suggested skills

## Before

```
$ dex init -t does-not-exist
Error: template 'does-not-exist' not found. Available: default, dabs-package, ...
```
No way to discover templates without triggering an error.

## After

```
$ dex templates list

Available templates:

  ●  built-in
    dabs-aiagent    Databricks Asset Bundle AI agent — mlflow.pyfunc, model serving
    dabs-etl        Databricks Asset Bundle ETL project with DLT pipeline and Autoloader
    dabs-ml         Databricks Asset Bundle ML project with MLflow training and serving
    ...

$ dex templates show dabs-ml

Template: dabs-ml
  Description:  Databricks Asset Bundle ML project ...
  Variables:
    project_name   Project name (required)
    python_version Python version  default: "3.12"
    ...
```

🤖 Generated with [Claude Code](https://claude.com/claude-code)

---------

Co-authored-by: Claude Sonnet 4.6 <noreply@anthropic.com>

## Changed files

- `TASKS.md`
- `crates/dex-cli/src/commands/init.rs`
- `crates/dex-cli/src/commands/mod.rs`
- `crates/dex-cli/src/commands/templates.rs`
- `crates/dex-cli/src/main.rs`
- `docs/internal/prd-dex-lock.md`
- `docs/internal/shipped/prd-templates.md`
- `docs/internal/shipped/prd-templating-strategy.md`

## Relationships

- **co-occurrence** → [[2f1e593-add-dex-skills-system-agent-skill-pack]] (3 shared files)
- **co-occurrence** → [[30690b9-port-dex-to-pure-rust-single-binary-21]] (3 shared files)
- **co-occurrence** → [[6cbcd49-add-prd-for-ai-ready-scaffolding-context-map]] (2 shared files)
- **resolved-by** → `#40` _(this commit)_
