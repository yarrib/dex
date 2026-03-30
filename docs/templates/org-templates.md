# Org Template Registries

Teams can publish their own templates and share them across the org via a shared directory
or git repository. Users reference them the same way as built-in templates.

## How it works

1. Create a directory of templates following the [authoring guide](authoring.md)
2. Distribute the directory (git repo, shared filesystem, etc.)
3. Users point dex at the directory via `~/.config/dex/config.toml` or `--template-dir`

No package manager, no Python, no runtime required.

---

## 1. Create the template directory

```
acme-dex-templates/
├── acme-etl/
│   ├── template.toml
│   └── files/
│       ├── dex.toml.j2
│       ├── pyproject.toml.j2
│       ├── README.md.j2
│       └── src/
│           └── {{ project_name }}/
│               └── pipeline.py.j2
└── acme-ml/
    ├── template.toml
    └── files/
```

Templates follow the same format as built-in templates. See the [authoring guide](authoring.md).

---

## 2. Distribute via git

Host the templates in a git repository:

```bash
git clone https://github.com/acme/acme-dex-templates
```

---

## 3. Configure dex to find the templates

### Per-user (global)

Add the templates directory to `~/.config/dex/config.toml`:

```toml
[templates]
paths = ["~/acme-dex-templates"]
```

Now all `dex` commands on this machine can use the org templates.

### Per-project

Place templates in a `templates/` directory at the project root:

```
my-project/
├── dex.toml
└── templates/
    └── acme-etl/
        ├── template.toml
        └── files/
```

dex discovers them automatically.

### One-off (via CLI flag)

```bash
dex init --template-dir ~/acme-dex-templates --template acme-etl --dir my_pipeline
```

---

## User experience

After configuring the templates path, users have all built-in templates plus the org templates:

```bash
dex init --template default       # built-in
dex init --template dabs-package  # built-in
dex init --template acme-etl      # org template
dex init --template acme-ml       # org template
```

If an org template name conflicts with a built-in, the org template takes precedence.

---

## Versioning org templates

Pin a specific commit or tag in your team's setup instructions:

```bash
git clone --branch v1.2.0 https://github.com/acme/acme-dex-templates ~/acme-dex-templates
```

Or check out the templates directory as a git submodule in your project repos.

## Monorepo pattern

If templates live in a monorepo alongside other tooling:

```toml
# ~/.config/dex/config.toml
[templates]
paths = ["~/acme-platform/dex-templates"]
```
