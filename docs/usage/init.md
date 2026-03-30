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
| `--preset` | — | Pre-fill variables from a named preset profile |
| `--presets-file` | — | Path to a presets TOML file |
| `--standards` | — | Path to a standards TOML file for variable pre-fills |

## Examples

```bash
# Scaffold into a new directory, prompting for all variables
dex init --template dabs-package --dir my_project

# Non-interactive: use all defaults
dex init --template dabs-package --no-prompt --dir my_project

# Pre-fill variables from a preset profile
dex init --template dabs-package --preset ml-project --dir my_project

# Use a team standards file
dex init --template default --standards ./org-standards.toml --dir my_package
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

Lists all built-in templates with names and descriptions.
