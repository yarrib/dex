# Template Authoring Guide

A dex template is a directory with a manifest (`template.toml`) and a `files/` subtree of Jinja2 template files. Templates are embedded into the dex binary at compile time via `include_dir`.

## Directory layout

```
templates/<template-name>/
├── template.toml          # manifest: metadata, variables, file rules
└── files/                 # files to render and write
    ├── pyproject.toml.j2  # .j2 = rendered through Jinja2
    ├── README.md.j2
    └── src/
        └── {{ project_name }}/   # directory names can use variables too
            └── __init__.py
```

## template.toml reference

### `[template]` — metadata

```toml
[template]
name = "my-template"          # unique identifier, used with dex init --template
description = "Short description shown in dex init --help"
version = "0.1.0"
min_dex_version = "0.1.0"    # minimum dex version required
```

### `[[variables]]` — input variables

Each variable becomes a prompt in `dex init` and a value available in templates.

```toml
[[variables]]
name = "project_name"         # variable name, referenced in templates as {{ project_name }}
prompt = "Project name"       # text shown to the user
type = "string"               # string | bool | choice
required = true
validate = "^[a-z][a-z0-9_]*$"  # optional regex; applied to string variables
```

**Variable types:**

=== "string"

    ```toml
    [[variables]]
    name = "author"
    prompt = "Author name"
    type = "string"
    required = false
    default = "me"
    ```

=== "bool"

    ```toml
    [[variables]]
    name = "include_notebook"
    prompt = "Include exploration notebook?"
    type = "bool"
    default = true     # rendered as true/false
    required = false
    ```

=== "choice"

    ```toml
    [[variables]]
    name = "python_version"
    prompt = "Python version"
    type = "choice"
    choices = ["3.12", "3.11"]
    default = "3.12"
    required = false
    ```

**All variable fields:**

| Field | Required | Description |
|---|---|---|
| `name` | yes | Variable identifier. Referenced in templates as `{{ name }}`. |
| `prompt` | yes | Text shown when prompting the user. |
| `type` | yes | `string`, `bool`, or `choice`. |
| `required` | yes | If `true`, no default is accepted. |
| `default` | no | Value used with `--no-prompt` or when the user presses Enter. |
| `choices` | choice only | List of accepted values. |
| `validate` | string only | Regex the value must match. |

### `[[files]]` — conditional file rules

Use `[[files]]` to include or exclude entire directory trees based on a variable.

```toml
[[files]]
src = "notebooks/"       # path relative to files/
condition = "include_notebook"   # include only if this bool variable is true

[[files]]
src = "resources/"
condition = "include_job"
```

If no `[[files]]` entry exists for a path, it is always included.

## Writing template files

Template files use [Jinja2](https://jinja.palletsprojects.com/) syntax, rendered by [minijinja](https://github.com/mitsuhiko/minijinja) in Rust. Use the `.j2` extension for any file that needs rendering.

```python
# src/{{ project_name }}/main.py.j2
"""{{ project_name }} — entry point."""


def main() -> None:
    print("Hello from {{ project_name }}")


if __name__ == "__main__":
    main()
```

**Variable substitution in filenames:**

Directory and file names can also contain variable references. The engine substitutes them before writing.

```
files/src/{{ project_name }}/__init__.py   →   src/my_project/__init__.py
```

**Conditionals:**

```toml
# pyproject.toml.j2
[project]
name = "{{ project_name }}"
requires-python = ">={{ python_version }}"
{% if use_serverless %}
# serverless config here
{% endif %}
```

**Loops:**

```
{% for dep in extra_deps %}
"{{ dep }}",
{% endfor %}
```

## Building and testing

After adding or editing a template, rebuild to embed it in the binary:

```bash
make build
```

Then test with:

```bash
dex init --template my-template --dir /tmp/test-output
dex init --template my-template --no-prompt --dir /tmp/test-output-defaults
```

Inspect the output to verify files were rendered correctly.

## Tips

- **Non-template files** (files without `.j2`) are copied verbatim — useful for binary assets or files where Jinja2 syntax would conflict.
- **Validate regex early.** Use `validate` on `project_name` to enforce naming conventions before the user gets to see broken output.
- **Use `bool` variables for optional sections.** Combine with `[[files]]` to omit entire directories, and with `{% if %}` inside templates to omit sections within a file.
- **Keep defaults sensible.** Templates should work correctly with `--no-prompt`, so every optional variable needs a reasonable default.
