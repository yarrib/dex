# dabs-platform

A single, profile-driven [dex](https://github.com/yarrib/dex) template for Databricks
Asset Bundle (DABs) projects. One baseline, optional components gated by boolean
variables, and four presets that select the components for a given deployment profile.

## Usage

```bash
# Data engineering (DLT pipeline + data-quality monitor)
dex init -t dabs-platform --preset data_eng \
    --presets-file templates/dabs-platform/presets.toml --dir my_pipeline

# MLOps (experiment + model, realtime serving, inference monitor)
dex init -t dabs-platform --preset mlops \
    --presets-file templates/dabs-platform/presets.toml --dir my_model

# Standalone agent starter
dex init -t dabs-platform --preset agents \
    --presets-file templates/dabs-platform/presets.toml --dir my_agent

# Extraction agents (monorepo-coupled, realtime + batch)
dex init -t dabs-platform --preset extraction_agents \
    --presets-file templates/dabs-platform/presets.toml --dir my_extractor
```

Without `--preset`/`--no-prompt`, dex prompts for every variable (power-user path).

## Profiles → components

| Component (gate) | data_eng | mlops | agents | extraction_agents |
|---|:-:|:-:|:-:|:-:|
| Base job (always) | ✅ | ✅ | ✅ | ✅ |
| Pipeline (`include_pipeline`) | ✅ | | | |
| Experiment + model (`include_experiment`) | | ✅ | ✅ | ✅ |
| Realtime serving (`include_realtime`) | | ✅ | ✅ | ✅ |
| Batch inference (`include_batch`) | | | | ✅ |
| Observability (`include_observability`) | ✅ | ✅ | ✅ | ✅ |
| Agent wrapper + eval (`include_agent`) | | | ✅ | ✅ |
| Databricks App (`include_app`) | | | | |
| Lakebase (`include_lakebase`) | | | | |
| React frontend (`include_react`) | | | | |
| `shared_code_source` | none | none | none | workspace |

`include_app`, `include_lakebase`, and `include_react` are wired and testable but not
enabled by any of these four profiles (they belong to a future user-facing-agent profile).

### Gating style

`[[files]]` conditions and `{% raw %}{% if %}{% endraw %}` blocks test a variable for
truthiness. Each preset lists only the bools it wants **on** (as `"true"`) and omits the
rest; omitted bools fall back to the template default `false`, which is the correct off
state.

> Historically a preset value was applied as a raw string, and any non-empty string —
> including `"false"` — was truthy, so a `"false"` bool would wrongly *enable* its
> component. **dex ≥ 0.6 coerces typed pre-fills**, so an explicit `include_x = "false"`
> now gates OFF correctly too. Omission is still the recommended style — it keeps presets
> minimal — but `"false"` is no longer a footgun.

## Org distribution

Ship this template to your team via an org registry. In `~/.config/dex/config.toml`:

```toml
[[templates.remotes]]
name = "dabs-platform"
url = "https://github.com/your-org/dex-templates.git"
subdir = "dabs-platform"
ref = "v1.0.0"            # pin to a tag/commit for reproducibility
```

Put org-wide constants in `~/.config/dex/standards.toml` so they pre-fill without a
preset (higher priority than presets, lower than an answers file):

```toml
python_version = "3.12"
author = "data-platform"
team_group = "data-platform"
central_ci_ref = "your-org/central-workflows/.github/workflows/ci.yml@v3"
central_cd_ref = "your-org/central-workflows/.github/workflows/cd.yml@v3"
```

Copy `presets.toml` to `~/.config/dex/presets.toml` so `--preset` resolves without
`--presets-file`.

## Notes / gaps

- **`databricks bundle validate` is not run by dex** — the emitted YAML follows the
  Azure DABs resource conventions, but validate against your workspace's CLI version.
- **Lakebase:** `databricks bundle destroy` does not tear down Lakebase (Postgres)
  projects — delete them via the CLI. Requires databricks CLI ≥ 0.287.0.
- Setup (`uv sync`, and `npm install` for the React frontend) is documented in the
  generated project README; dex runs `uv sync` on scaffold via `[on_success]`.
