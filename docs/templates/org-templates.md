# Org Template Registries

Teams can publish their own templates and share them across the org via a shared directory
or git repository. Users reference them the same way as built-in templates.

## How it works

1. Create a directory of templates following the [authoring guide](authoring.md)
2. Distribute the directory (git repo, shared filesystem, etc.)
3. Users point dex at the templates via `~/.config/dex/config.toml`

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

## 2. Distribute

Host the templates in a git repository, or sync the directory to a shared
location your team can reach.

---

## 3. Configure dex to find the templates

### Per-user, local directory

Point `~/.config/dex/config.toml` at a directory of templates. `dir` is a
single path string (not a list, and not `paths`):

```toml
[templates]
dir = "~/acme-dex-templates"
```

Now all `dex` commands on this machine can use the org templates.

### Per-user, remote git repo

Let dex own the clone. It clones each repo into its cache
(`~/.cache/dex/templates/<name>`) on first use and `git pull`s it on later
runs, so engineers never clone or update by hand:

```toml
[[templates.remotes]]
name = "acme-templates"
url  = "https://github.com/acme/acme-dex-templates.git"
ref  = "main"   # optional: branch, tag, or commit
```

You can list `dir` and one or more `[[templates.remotes]]` together.

### Per-project

Keep templates inside a project repo and opt in from its `dex.toml`. dex does
not auto-scan `./templates/`, so name the directory explicitly:

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

With a remote, pin a tag or commit via `ref` so everyone resolves the same
templates:

```toml
[[templates.remotes]]
name = "acme-templates"
url  = "https://github.com/acme/acme-dex-templates.git"
ref  = "v1.2.0"
```

With a local `dir`, pin in your team's setup instructions instead:

```bash
git clone --branch v1.2.0 https://github.com/acme/acme-dex-templates ~/acme-dex-templates
```

## Monorepo pattern

If templates live in a monorepo alongside other tooling, point `dir` at the
subdirectory:

```toml
# ~/.config/dex/config.toml
[templates]
dir = "~/acme-platform/dex-templates"
```

---

## A runnable example

[`examples/`](../../examples/README.md) contains a complete worked setup: a
sample `acme-dex-templates` repo with a working `acme-etl` template, plus
example `config.toml`, `standards.toml`, `presets.toml`, and project `dex.toml`
files you can copy.
