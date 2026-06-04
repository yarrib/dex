# Examples: setting up dex for your org

Worked, copy-pasteable examples for rolling dex out to a team — sharing
templates, standardizing pass-throughs and tasks, and pre-filling variables so
engineers go from `dex init` to a deployable project in seconds.

Everything here uses a fictional **Acme** org and the exact config syntax dex
understands today. You can run it as-is.

```
examples/
├── acme-dex-templates/        # a shared org templates repo (one template: acme-etl)
│   └── acme-etl/
│       ├── template.toml      # manifest: metadata, variables, file rules, on_success
│       └── files/             # .j2 files are rendered; others copied verbatim
└── config/
    ├── config.toml            # user config  → ~/.config/dex/config.toml
    ├── standards.toml         # flat, org-wide variable pre-fills
    ├── presets.toml           # named bundles of pre-fills (--preset)
    └── dex.toml               # project config → checked into a repo
```

## The three ways to standardize an org

You can use any combination of these. They are independent.

| Goal | Mechanism | Setup |
|---|---|---|
| Share project **scaffolds** | Templates in a dir or git repo | `[templates]` in user config |
| Share **commands & tasks** | `dex.toml` checked into repos | nothing per-user |
| Skip repetitive **prompts** | Standards / presets files | `--standards` / `--preset` |

---

## 1. Point dex at your org templates

A "templates directory" is any directory whose immediate subdirectories are
templates (each with a `template.toml`). `examples/acme-dex-templates/` is one.

Each engineer adds **one** of these to `~/.config/dex/config.toml`
(see [`config/config.toml`](config/config.toml)):

**Local directory** — a single path (note: `dir`, a string, not `paths`):

```toml
[templates]
dir = "~/acme-dex-templates"
```

**Remote git repo** — dex clones it to its cache on first use and pulls it on
later runs, so nobody clones or updates by hand:

```toml
[[templates.remotes]]
name = "acme-templates"
url  = "https://github.com/acme/acme-dex-templates.git"
ref  = "v1.2.0"        # optional: branch, tag, or pinned release
```

Either way, org templates appear right alongside the built-ins:

```bash
dex templates list
dex init --template acme-etl --dir my_pipeline
```

> There is **no** `--template-dir` flag and **no** automatic `./templates/`
> discovery. To use a template that lives inside a project (a monorepo, say),
> opt in from that project's `dex.toml`:
>
> ```toml
> [templates]
> dir = "templates"
> ```

### Try it now, no config edits

This repo's own example template renders out of the box:

```bash
# from a scratch dir whose dex.toml points at the example templates
mkdir -p /tmp/acme && cd /tmp/acme
printf '[project]\nname = "scratch"\n\n[templates]\ndir = "%s/examples/acme-dex-templates"\n' \
  "$(git -C path/to/dex rev-parse --show-toplevel)" > dex.toml

dex init --template acme-etl --no-prompt --dir /tmp/acme/out
```

You'll get a full pipeline: `pyproject.toml`, a DLT `pipeline.py`, a bundle
`databricks.yml`, a pre-wired `dex.toml`, and (because `include_notebook`
defaults to true) a `notebooks/` directory.

---

## 2. Standardize commands & tasks with a project `dex.toml`

The lowest-friction option — no templates needed. Check a `dex.toml`
(see [`config/dex.toml`](config/dex.toml)) into a repo and everyone who clones
it inherits the same pass-throughs, tasks, and profiles:

```toml
[passthrough.db]
command = "databricks"          # dex db ...  → databricks ...

[tasks.deploy-dev]
command    = "databricks bundle deploy --target dev"
depends_on = ["test"]            # runs `test` first
```

```bash
dex db clusters list      # → databricks clusters list
dex run deploy-dev        # runs test, then deploys
```

The `acme-etl` template ships exactly this `dex.toml`, so scaffolded projects
get the team's commands automatically.

---

## 3. Pre-fill variables so prompts disappear

**Standards** ([`config/standards.toml`](config/standards.toml)) — a flat set of
org-wide constants applied to every scaffold:

```bash
dex init --template acme-etl --standards examples/config/standards.toml --dir my_pipeline
```

Put it at `~/.config/dex/standards.toml` to apply it automatically.

**Presets** ([`config/presets.toml`](config/presets.toml)) — named bundles you
pick per run:

```bash
dex init --template acme-etl --preset etl \
  --presets-file examples/config/presets.toml --dir my_pipeline
```

Pre-fill precedence, lowest → highest:

1. Template default (`template.toml`)
2. `--preset` profile
3. `--standards` values
4. `--answers` (a saved answers file)
5. Interactive answer

A key only fills a prompt if its name matches a template variable; anything
unmatched is ignored.

---

## Full onboarding, end to end

```bash
# 1. Install dex
curl -sSf https://raw.githubusercontent.com/yarrib/dex/main/install.sh | sh

# 2. Point dex at the org templates + defaults (once)
mkdir -p ~/.config/dex
cp examples/config/config.toml   ~/.config/dex/config.toml
cp examples/config/standards.toml ~/.config/dex/standards.toml

# 3. Scaffold — no prompts thanks to standards + defaults
dex init --template acme-etl --dir my_pipeline
cd my_pipeline && dex run deploy-dev
```

## See also

- [Building Org Templates](../docs/templates/org-templates-guide.md) — author a template from scratch
- [Org Template Registries](../docs/templates/org-templates.md) — sharing options reference
- [Extending dex](../docs/extending.md) — pass-throughs and custom templates
- [Template Authoring](../docs/templates/authoring.md) — full `template.toml` reference
