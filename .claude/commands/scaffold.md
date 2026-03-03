Scaffold a new dex project using `dex init`.

Usage:
```bash
dex init --template default --dir /path/to/new-project
```

Available templates:
```bash
dex init --help          # shows options
```

Non-interactive (use all defaults):
```bash
dex init --template default --dir /tmp/my-project --no-prompt
```

What happens:
1. dex resolves the template from embedded templates (or a custom dir)
2. Variables are collected interactively (or from defaults with `--no-prompt`)
3. `scaffold_project` in `dex-core` renders Jinja2 templates and writes files
4. Output lists all files created

Template locations:
- Embedded: `templates/` (compiled into the binary via `include_dir`)
- Custom: set `templates_dir` in `create_cli()` or `dex.toml`

To add a new template variable, edit `templates/<name>/template.toml`.
Template files use `.j2` extension and Jinja2 syntax.
