---
sha: ff57dbb95e551ec159c9a18c313b28e13c1a55c6
short_sha: ff57dbb
author: yarrib
date: 2026-06-04
class: [Decision]
area: Foundation & Architecture
tags: [#decision]
---

# [Decision] docs: add runnable org-setup examples and fix template config syntax (#56)

**Commit:** `ff57dbb` · **Author:** yarrib · **Date:** 2026-06-04 · **Area:** Foundation & Architecture

_No extended commit description._

## Changed files

- `README.md`
- `docs/SPEC.md`
- `docs/extending.md`
- `docs/index.md`
- `docs/templates/org-templates-guide.md`
- `docs/templates/org-templates.md`
- `examples/README.md`
- `examples/acme-dex-templates/acme-etl/files/.gitignore`
- `examples/acme-dex-templates/acme-etl/files/README.md.j2`
- `examples/acme-dex-templates/acme-etl/files/databricks.yml.j2`
- `examples/acme-dex-templates/acme-etl/files/dex.toml.j2`
- `examples/acme-dex-templates/acme-etl/files/notebooks/exploration.py.j2`
- `examples/acme-dex-templates/acme-etl/files/pyproject.toml.j2`
- `examples/acme-dex-templates/acme-etl/files/resources/{{ project_name }}_pipeline.yml.j2`
- `examples/acme-dex-templates/acme-etl/files/src/{{ project_name }}/__init__.py.j2`
- `examples/acme-dex-templates/acme-etl/files/src/{{ project_name }}/pipeline.py.j2`
- `examples/acme-dex-templates/acme-etl/files/tests/__init__.py`
- `examples/acme-dex-templates/acme-etl/files/tests/test_{{ project_name }}.py.j2`
- `examples/acme-dex-templates/acme-etl/template.toml`
- `examples/config/config.toml`
- _…and 3 more_

## Relationships

- **influenced-by** → [[b8fa631-add-prd-for-snowflake-templates-35]] (Foundation & Architecture)
- **co-occurrence** → [[e3efd53-rewrite-all-docs-for-rust-binary-architecture-25]] (6 shared files)
- **co-occurrence** → [[31c18f8-feat-documentation-12]] (3 shared files)
- **co-occurrence** → [[0e067c2-add-workflow-dispatch-to-docs-deploy-workflow]] (3 shared files)
- **resolved-by** → `#56` _(this commit)_
