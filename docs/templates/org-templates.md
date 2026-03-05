# Org Template Registries

Teams can publish their own template registries and surface them through a custom org CLI built with `create_cli()`. Users get `dex init --template <your-template>` alongside all built-in templates, under your own CLI name.

## How it works

1. Create a Python package containing your templates
2. Wire the templates directory into `create_cli()` via `templates_dir`
3. Distribute the org CLI package via internal PyPI, Artifactory, or Git URL
4. Users install the org CLI once and get all your org templates

---

## 1. Create the template registry package

```
acme-dex/
├── pyproject.toml
├── acme_dex/
│   ├── __init__.py
│   ├── cli.py              # org CLI entry point
│   └── templates/          # your org templates
│       ├── acme-etl/
│       │   ├── template.toml
│       │   └── files/
│       └── acme-ml/
│           ├── template.toml
│           └── files/
```

Templates follow the same format as built-in templates. See the [authoring guide](authoring.md).

## 2. Wire templates into create_cli()

```python
# acme_dex/cli.py
from pathlib import Path
from dex.ext import create_cli, passthrough

_TEMPLATES_DIR = Path(__file__).parent / "templates"

cli = create_cli(
    name="acme-dex",
    templates_dir=_TEMPLATES_DIR,
    passthroughs=[
        passthrough("db", "databricks", "Databricks CLI"),
    ],
)
```

The `templates_dir` path is resolved relative to the installed package, so it works correctly regardless of where users install it.

## 3. Package and distribute

```toml
# pyproject.toml
[project]
name = "acme-dex"
version = "0.1.0"
dependencies = ["dex>=0.1.0"]

[project.scripts]
acme-dex = "acme_dex.cli:cli"

[tool.setuptools.package-data]
acme_dex = ["templates/**/*"]
```

!!! important "Include template files in the package"
    Template files must be declared in `package-data` (setuptools) or `include` (hatch/flit) so they are bundled into the wheel. Without this, the `templates/` directory will be missing after install.

### Distribute via internal PyPI

```bash
# Build and publish
uv build
uv publish --index https://pypi.internal.acme.com/

# Users install with
uv tool install acme-dex --index https://pypi.internal.acme.com/
```

### Distribute via Git URL ("pull and attach")

No internal PyPI needed — users install directly from the repo:

```bash
uv tool install "acme-dex @ git+https://github.com/acme/acme-dex"
```

Pin a specific release:

```bash
uv tool install "acme-dex @ git+https://github.com/acme/acme-dex@v1.2.0"
```

---

## User experience

After installing the org CLI, users have all built-in dex templates plus your org templates:

```bash
acme-dex init --template default        # built-in
acme-dex init --template dabs-package   # built-in
acme-dex init --template acme-etl       # org template
acme-dex init --template acme-ml        # org template
```

If an org template name conflicts with a built-in, the org template takes precedence.

---

## Versioning org templates

Tag releases and pin the version in your team's install instructions:

```bash
# Install a specific version
uv tool install "acme-dex==1.2.0" --index https://pypi.internal.acme.com/

# Upgrade
uv tool upgrade acme-dex --index https://pypi.internal.acme.com/
```

## Monorepo pattern

If your templates live in a monorepo alongside other tooling, point `templates_dir` at the right subdirectory:

```python
_TEMPLATES_DIR = Path(__file__).parent.parent / "platform" / "dex-templates"

cli = create_cli(name="acme-dex", templates_dir=_TEMPLATES_DIR)
```
