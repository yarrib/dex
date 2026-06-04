# Extending dex

dex is designed to be extended by teams through configuration and custom templates.
No code is required — everything is driven by `dex.toml` and a templates directory.

## Pass-through commands

The most common extension: expose an external CLI as a `dex` subcommand, forwarding
all arguments and inheriting stdin/stdout/stderr for full interactivity.

Add pass-throughs to `dex.toml` at your project root:

```toml
[passthrough.db]
command = "databricks"
description = "Databricks CLI"

[passthrough.tf]
command = "terraform"
description = "Terraform"

[passthrough.az]
command = "az"
description = "Azure CLI"
```

Your team now has:

```bash
dex db clusters list               # → databricks clusters list
dex tf plan                        # → terraform plan
dex az account show                # → az account show
```

Pass-throughs appear in `dex --help` and forward `--help` to the target command.

## Custom templates

Add custom templates to a directory and tell dex where to find them.

### Per-user (global)

In `~/.config/dex/config.toml`, point at a local directory of templates
(a single path string under `dir`):

```toml
[templates]
dir = "~/acme-dex-templates"
```

Or have dex clone and update one or more git repos for you:

```toml
[[templates.remotes]]
name = "acme-templates"
url  = "https://github.com/acme/acme-dex-templates.git"
ref  = "v1.2.0"   # optional: branch, tag, or commit
```

### Per-project

Keep templates inside a project (e.g. a monorepo) by opting in from that
project's `dex.toml`. dex does not auto-scan `./templates/`, so name the
directory explicitly:

```
my-project/
├── dex.toml          # [templates] dir = "templates"
└── templates/
    └── acme-etl/
        ├── template.toml
        └── files/
```

```toml
# dex.toml
[templates]
dir = "templates"
```

Then use them like any built-in template:

```bash
dex init --template acme-etl --dir my_pipeline
```

See the [Template Authoring Guide](templates/authoring.md) for the full template format,
and [Org Template Registries](templates/org-templates.md) for how to share templates
across a team. For a complete, runnable org setup — a sample templates repo plus
config, standards, and presets files — see [`examples/`](../examples/README.md).

## Org-wide dex.toml

For teams, check a `dex.toml` into your project repos with shared pass-throughs
and task definitions:

```toml
[project]
name = "my-project"

[passthrough.db]
command = "databricks"
description = "Databricks CLI"

[tasks.deploy-dev]
command = "databricks bundle deploy --target dev"
description = "Deploy to dev"

[tasks.deploy-prod]
command = "databricks bundle deploy --target prod"
description = "Deploy to prod"
```

All engineers on the project get the same commands — no individual setup required
beyond having `databricks` on their `PATH`.
