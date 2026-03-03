# dex init

Scaffold a new project from a template.

## Synopsis

```
dex init [OPTIONS] [DIRECTORY]
```

## Options

| Option | Default | Description |
|---|---|---|
| `--template`, `-t` | `default` | Template to scaffold from |
| `--dir`, `-d` | `.` | Target directory |
| `--no-prompt` | — | Use all defaults, skip interactive prompts |

## Examples

```bash
# Scaffold into current directory, prompting for all variables
dex init --template dabs-package

# Scaffold into a new directory
dex init --template dabs-package --dir my_project

# Non-interactive: use all defaults
dex init --template dabs-package --no-prompt --dir my_project
```

## Interactive prompts

When you run `dex init` without `--no-prompt`, it asks for each variable defined in the template's manifest. For example, `dabs-package` asks:

```
Project name [my_project]:
Python version (3.12, 3.11) [3.12]:
Include exploration notebook? [Y/n]:
Include job definition? [Y/n]:
Use serverless compute? [y/N]:
```

## Listing available templates

```bash
dex init --help
```

Use the MCP server or `dex` source to see all built-in templates, or run:

```python
from dex._core import list_embedded_templates
for t in list_embedded_templates():
    print(t.name, "-", t.description)
```
