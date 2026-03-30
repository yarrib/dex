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

In `~/.config/dex/config.toml`:

```toml
[templates]
paths = ["~/acme-dex-templates"]
```

### Per-project

Place templates in a `templates/` directory at the project root. dex discovers
them automatically alongside built-in templates.

```
my-project/
├── dex.toml
└── templates/
    └── acme-etl/
        ├── template.toml
        └── files/
```

Then use them like any built-in template:

```bash
dex init --template acme-etl --dir my_pipeline
```

See the [Template Authoring Guide](templates/authoring.md) for the full template format,
and [Org Template Registries](templates/org-templates.md) for how to share templates
across a team.

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
