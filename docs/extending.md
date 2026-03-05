# Extending dex — Org CLIs

dex is designed to be wrapped by teams into their own internal CLI. You get all the built-in dex commands (`init`, `agent new`, `mcp serve`) plus the ability to add custom commands, custom templates, and pass-throughs to other tools — all under your own CLI name.

## Quick example

```python
# acme_dex/cli.py
from dex.ext import create_cli, passthrough

cli = create_cli(
    name="acme-dex",
    passthroughs=[
        passthrough("db", "databricks", "Databricks CLI"),
        passthrough("tf", "terraform", "Terraform"),
    ],
)

@cli.command()
def deploy():
    """Deploy the current bundle to dev."""
    import subprocess
    subprocess.run(["databricks", "bundle", "deploy"], check=True)
```

```toml
# pyproject.toml
[project.scripts]
acme-dex = "acme_dex.cli:cli"
```

Your users now have:

```bash
acme-dex init --template dabs-package   # built-in
acme-dex deploy                         # your custom command
acme-dex db clusters list               # pass-through to databricks
acme-dex tf plan                        # pass-through to terraform
```

## `create_cli()`

The factory that wires everything together.

```python
from dex.ext import create_cli

cli = create_cli(
    name="acme-dex",           # CLI name shown in --help
    templates_dir="./templates",  # path to your org's custom templates (optional)
    passthroughs=[...],        # list of PassthroughSpec (optional)
)
```

| Parameter | Type | Description |
|---|---|---|
| `name` | `str` | CLI name. Defaults to `"dex"`. |
| `templates_dir` | `str \| Path \| None` | Directory of custom templates to add alongside built-ins. |
| `passthroughs` | `list[PassthroughSpec]` | External CLIs to expose as subcommands. |

`create_cli()` returns a `DexGroup` (a `click.Group` subclass). Add commands to it exactly as you would with click.

## Pass-through commands

Pass-throughs proxy a subcommand directly to an external CLI, forwarding all arguments and inheriting stdin/stdout/stderr for full interactivity.

```python
from dex.ext import create_cli, passthrough

cli = create_cli(
    name="acme-dex",
    passthroughs=[
        passthrough("db", "databricks", "Databricks CLI"),
    ],
)
```

`passthrough(name, command, description)`:

| Parameter | Description |
|---|---|
| `name` | Subcommand name in your CLI (`acme-dex db ...`) |
| `command` | Executable to invoke (`databricks`) |
| `description` | Help text shown in `--help` |

You can also construct `PassthroughSpec` directly:

```python
from dex.ext import PassthroughSpec

spec = PassthroughSpec(name="db", command="databricks", description="Databricks CLI")
```

## Custom commands

After calling `create_cli()`, decorate commands onto the returned group:

```python
cli = create_cli(name="acme-dex")

@cli.command()
@click.option("--target", default="dev")
def deploy(target: str):
    """Deploy the current bundle."""
    import subprocess
    subprocess.run(["databricks", "bundle", "deploy", "--target", target], check=True)
```

Commands can also be added programmatically:

```python
cli.add_command(my_command_func, name="my-command")
```

## Custom templates

Point `templates_dir` at a directory of templates following the [authoring guide](templates/authoring.md). Users can then use them with `dex init --template <name>`:

```python
cli = create_cli(
    name="acme-dex",
    templates_dir=Path(__file__).parent / "templates",
)
```

Your org templates live alongside the dex built-ins. If a name conflicts, your template wins.

## Distributing your org CLI

Package it as a standard Python package and distribute via your internal PyPI, Artifactory, or direct install:

```bash
# Install from internal PyPI
uv tool install acme-dex --index https://pypi.internal.acme.com/

# Or from a Git repo
uv tool install "acme-dex @ git+https://github.com/acme/acme-dex"
```

Your users run `acme-dex` directly — they never need to know about dex.

## `DexGroup`

`DexGroup` is the `click.Group` subclass returned by `create_cli()`. It handles pass-through resolution and can be used directly if you need lower-level control:

```python
from dex.ext import DexGroup

@click.group(cls=DexGroup, passthroughs={"db": db_spec})
def cli():
    """Acme internal tooling."""
```
