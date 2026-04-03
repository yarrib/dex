# PRD: Scaffolding Differentiation — Lessons from Cookiecutter, Baker, and CRA

**Status:** Planning
**Owner:** TBD

---

## 1. Purpose

This document analyzes three reference points in the scaffolding tool space —
**Cookiecutter**, **Baker**, and **create-react-app** — to identify what dex should
borrow, what it should avoid, and where it has genuine opportunities to differentiate.
All proposals are constrained to dex's architectural guardrails: 100% Rust, single
binary, no Python runtime, TOML config, dex-core/dex-cli layering.

---

## 2. Competitive Reference Points

### 2.1 Cookiecutter

**What it is:** Python CLI for scaffolding any project type from templates. The
de-facto standard in the Python/ML community. Templates are git repos or local
directories.

**Template format:**
- Variables defined in `cookiecutter.json` (a flat JSON dict of `variable: default`)
- Jinja2 used in file contents and file/directory names
- No native conditional file inclusion — done via post-gen hooks
- Pre/post hooks: Python scripts in a `hooks/` directory that run before/after generation

**Variable system:**
- String only by default; arrays render as select prompts
- No native `bool` type; convention is `"y"/"n"` strings
- No multi-select, no regex validation, no required-vs-optional distinction
- `--no-input` flag skips all prompts and uses defaults
- Replay: saves last answers in `.cookiecutter_replay/<template>.json`

**Template discovery:**
- GitHub topic `cookiecutter` — effectively a community convention, no real registry
- `--directory` flag for monorepo templates
- Template versioning via git tags/branches

**Distribution:**
- Requires Python and `pip install cookiecutter`
- Templates are just git repos — no signing, no versioning contract

**Key strengths:**
1. Familiar Jinja2 syntax — zero learning curve for Python developers
2. Hook system enables arbitrary post-generation logic
3. Massive community template ecosystem (GitHub topic has 5,000+ templates)
4. Replay support reduces friction for repeated use

**Key weaknesses:**
1. Python runtime required — breaks on Python version mismatches, virtualenvs
2. No native bool/multi types, no conditional files
3. No org-level pre-fill or standards mechanism
4. Hooks are arbitrary Python — security risk for untrusted templates
5. Template discovery is just GitHub search — no quality signal
6. No template versioning contract

---

### 2.2 Baker (cargo-generate / Rust scaffolding ecosystem)

**What it is:** `cargo-generate` is the primary Rust-native scaffolding tool in
this space. It generates Rust (and any language) projects from git template repos.
Baker-style tools in Rust generally follow the same pattern.

**Template format:**
- `cargo-generate.toml` manifest with `[template]` metadata and `[placeholders]`
- Placeholders: `prompt`, `type` (string, bool, choice), `default`, `regex`
- Jinja2 (Tera/MiniJinja) for file contents and paths
- Conditional includes via template logic in files, not in the manifest

**Variable system:**
- Stronger typing than cookiecutter: bool, string, choice are first-class
- `--define` flag for non-interactive use
- `favorites.toml` for pre-configured values (analogous to dex presets)
- Template variable names are validated against a regex

**Template discovery:**
- No registry — template is specified as a git URL
- `--favorite` flag for named shortcuts in `~/.cargo/generate/favorites.toml`
- No community index

**Distribution:**
- `cargo install cargo-generate` — requires Rust toolchain
- Templates are git repos; version pinned via `--tag` or `--branch`

**Key strengths:**
1. Native Rust binary — no runtime dependency
2. First-class bool/choice variable types
3. Git URL + tag for template versioning/pinning
4. `favorites.toml` for named shortcut templates
5. `--define` for scripted/CI use

**Key weaknesses:**
1. No conditional file inclusion in manifest (push logic into Jinja2 conditionals)
2. No org-wide standards/governance mechanism
3. No template registry — pure git URL, no discovery
4. No post-scaffold hooks beyond what the template can encode
5. No domain focus — completely generic

---

### 2.3 create-react-app (CRA)

**What it is:** The official React project bootstrapper (now deprecated; succeeded by
Vite, Next.js). One of the most influential scaffolding tools ever built — peak usage
~100k weekly downloads. Its design choices shaped an entire generation of tools.

**Template format:**
- No variable system — zero prompts by default
- Project name is the only required argument
- `--template` flag selects a template, distributed as npm packages (`cra-template-*`)
- Templates must conform to a strict directory contract

**UX philosophy:**
- **Zero-config** is the core value prop: the tool does one thing and it works
- Post-scaffold: automatically runs `npm install` — project is ready to run immediately
- "You don't need to know webpack exists"

**Template discovery:**
- npm registry search for `cra-template-*` packages
- Quality signal: npm download counts

**Distribution:**
- `npx create-react-app` — no install required (npx caches)
- Templates published as versioned npm packages

**Key strengths:**
1. Zero-friction happy path — one command, no decisions, ready to run
2. Auto-installs dependencies — project works before you open your editor
3. npm registry gives quality signal and versioning for templates
4. Template contract is strict enough that tools can reason about the output

**Key weaknesses:**
1. Too opinionated — `eject` was a hack that proved the model was wrong
2. No escape hatch before ejecting: hidden config was a footgun
3. Deprecated because the abstraction leaked: webpack complexity always surfaced eventually
4. No variable system at all — every variation required a new template

---

## 3. Synthesis

### 3.1 Table Stakes (all three do this, dex must too)

| Feature | Cookiecutter | Baker | CRA | dex today |
|---------|-------------|-------|-----|-----------|
| Jinja2 templating in file contents | ✅ | ✅ | — | ✅ |
| Template variable prompts | ✅ | ✅ | — | ✅ |
| `--no-prompt` / non-interactive mode | ✅ | ✅ | ✅ | ✅ |
| Template selection flag | ✅ | ✅ | ✅ | ✅ |
| Variable defaults | ✅ | ✅ | — | ✅ |
| Path interpolation (vars in filenames) | ✅ | ✅ | — | ✅ |
| Native bool/choice variable types | ❌ | ✅ | — | ✅ |

dex already meets or exceeds table stakes. Nothing to borrow here.

### 3.2 Where Each Tool Excels Uniquely

**Cookiecutter: Replay / idempotent re-use**
Saving the last prompt answers and enabling re-run is underrated. Useful for:
repeating a scaffold with slight variations, CI pipelines, and onboarding
("re-run this to see what you answered last time").

**Baker: Template version pinning**
Specifying `--tag v1.2.0` when loading a remote git template means scaffold
output is reproducible across time. Critical for org templates — teams need to
know which version of the template was used.

**CRA: Zero-config + post-scaffold activation**
Running `npm install` automatically so the project is immediately runnable was
the single highest-leverage UX decision CRA made. dex's equivalent: running
`uv sync` (or `pip install -e .`) after scaffold so the environment is ready.

### 3.3 Where All Three Fall Short (dex's opportunity)

| Gap | Current tools | dex's opportunity |
|-----|--------------|-------------------|
| Org-level governance | None | Standards + presets (already shipped) |
| Conditional file inclusion in manifest | None natively | `[[files]] condition =` (already shipped) |
| Domain-specific defaults | Generic | Databricks/ML-specific templates + defaults (already shipped) |
| AI-augmented generation | None | `dex agent new` Q&A → LLM generation (already shipped) |
| Single binary / no runtime | None | Rust binary, no Python/Node needed (already shipped) |
| Post-scaffold auto-activation | CRA only (Node) | **Gap: `uv sync` / post-scaffold hook** |
| Template version pinning | Baker (git tags) | **Gap: remote templates + version locking** |
| Replay / save last answers | Cookiecutter | **Gap: answer replay / `--replay` flag** |
| Template registry / discovery | Weak in all | **Gap: org template registry in config.toml** |
| Hook system | Cookiecutter (Python) | **Gap: safe shell hooks (no arbitrary exec by default)** |
| Template authoring tooling | None | **Gap: `dex template validate` / `dex template preview`** |

---

## 4. Proposed Features

The following proposals are ordered by user value vs. implementation cost.

---

### 4.1 Post-Scaffold Activation Hook (HIGH value, LOW cost)

**Problem:** After `dex init`, the user still has to manually run `uv sync` or
`pip install -e .` before the project works. CRA proved that auto-activation is
one of the highest-leverage UX moves a scaffolding tool can make.

**Proposal:** Add optional `[on_success]` to `template.toml`:

```toml
[template]
name = "default"

[on_success]
run = "uv sync"
message = "Project ready. Run: dex run test"
```

**Behavior:**
- After scaffold completes, dex prints a styled success summary
- If `run` is set, dex shows the command and asks: `Run setup now? [Y/n]`
- `--no-prompt` skips the confirmation and runs it automatically
- Command runs in the scaffolded directory via `std::process::Command`
- Failure is non-fatal: dex prints the error and suggests running manually

**Architectural fit:**
- `dex-core`: `ScaffoldResult` gains an `on_success: Option<OnSuccessConfig>` field
- `dex-cli/commands/init.rs`: reads `on_success` from result, prompts, spawns command
- No arbitrary code execution in core; core only returns config — CLI decides whether to run

**Constraints:**
- The command runs in a sandboxed manner: only in the new project directory
- No environment variable injection beyond what the shell provides
- Not the same as a full hook system — no pre-scaffold hooks in this proposal

---

### 4.2 Answer Replay (`--replay` / `--save-answers`) (HIGH value, LOW cost)

**Problem:** Users who scaffold the same template repeatedly (e.g., org teams
creating a new Databricks project each sprint) re-answer the same prompts every time.
There's no way to say "same as last time, except the project name."

**Proposal:** Save and load prompt answers as TOML files.

```bash
# Save answers from a scaffold session
dex init --template dabs-package --save-answers ~/last-dabs.toml

# Re-use saved answers (skips prompts where values are present)
dex init --template dabs-package --answers ~/last-dabs.toml

# Fully non-interactive replay
dex init --template dabs-package --answers ~/last-dabs.toml --no-prompt
```

**Answers file format** (`~/last-dabs.toml`):
```toml
# Saved by dex init on 2026-04-03
template = "dabs-package"
dex_version = "0.4.0"

[values]
project_name = "user_events"
python_version = "3.12"
include_notebook = true
use_serverless = false
```

**Pre-fill precedence update:**
```
defaults → standards → preset → answers file → interactive prompt
```

**Architectural fit:**
- `dex-core/src/config.rs`: add `AnswersFile` struct, `save_answers()`, `load_answers()`
- `dex-cli/commands/init.rs`: wire `--save-answers` and `--answers` flags
- Format is TOML (consistent with all other config in dex)
- Answers file is user-owned: dex writes it only when `--save-answers` is set

---

### 4.3 Remote Template Version Pinning (MEDIUM value, MEDIUM cost)

**Problem:** When a team uses a shared org template from a git repo, there's no
guarantee that `dex init --template my-org/dabs-etl` will produce the same output
next month. The template may have changed.

**Proposal:** Add `ref` (git tag or commit) to remote template config:

```toml
# ~/.config/dex/config.toml
[templates]
paths = ["~/acme-dex-templates"]

[[templates.remotes]]
name = "acme"
url  = "https://github.com/acme-corp/dex-templates.git"
ref  = "v1.4.0"         # pinned: tag, branch, or SHA
```

**Lockfile:** After `dex init` from a remote template, write to `dex.toml`:

```toml
[project]
name = "user-events"
template = "acme/dabs-etl"
template_ref = "v1.4.0"
template_sha = "abc123def456"   # resolved SHA at scaffold time
```

This gives teams a reproducible audit trail: "this project was scaffolded from
acme/dabs-etl at v1.4.0, commit abc123."

**Architectural fit:**
- `dex-core/src/template/registry.rs`: add remote fetch via `git2` or `std::process::Command git clone --depth 1`
- Remote templates are cached in `~/.cache/dex/templates/<name>/<ref>/`
- Cache invalidation: `dex init --update-templates` forces re-fetch
- `dex.toml` generation already happens in `init.rs`; add `template_ref` / `template_sha` fields

---

### 4.4 Template Authoring Tools (MEDIUM value, MEDIUM cost)

**Problem:** There are no tools to validate a `template.toml` or preview what a
template will produce. Template authors find bugs only at runtime.

**Proposal:** Two new subcommands under `dex template`:

```bash
dex template validate ./my-template
# Checks: manifest parses, all referenced files exist, variable names are valid,
# conditions reference declared variables, .j2 files compile without syntax errors

dex template preview ./my-template [--var project_name=foo] [--no-prompt]
# Renders the template to a temp dir and prints a tree of what would be created.
# --var overrides specific variables; --no-prompt uses all defaults.
# Cleans up temp dir after showing output.
```

**`dex template validate` output:**
```
Validating ./my-template...
  ✓ template.toml parsed
  ✓ 4 variables declared (project_name, python_version, include_notebook, use_serverless)
  ✓ 12 template files found
  ✓ All [[files]] conditions reference declared variables
  ✓ Jinja2 syntax valid in 8 .j2 files
  ✗ src/{{ project_slug }}/main.py.j2: variable 'project_slug' is not declared

1 error found.
```

**Architectural fit:**
- `dex-core`: add `validate_template(path: &Path) -> Vec<ValidationError>` to the template module
- `dex-cli/src/commands/template.rs`: new command file, register in `main.rs`
- Reuses existing `manifest.rs` parsing and `engine.rs` rendering logic
- Preview uses `scaffold()` with a temp dir from `tempfile` crate

---

### 4.5 Conditional Variable Visibility (LOW value, LOW cost)

**Problem:** A template may have 8 variables but only show 3 by default; the rest
are only relevant if a top-level flag is set. Currently all declared variables are
always prompted.

**Proposal:** Add optional `when` condition to variable declarations:

```toml
[variables]
project_name     = { prompt = "Project name", order = 1 }
include_serving  = { prompt = "Include model serving endpoint?", type = "bool", default = false, order = 2 }
serving_endpoint = { prompt = "Endpoint name", when = "include_serving", order = 3 }
```

**Behavior:** If `when` evaluates to false (variable reference or Jinja2 expression),
the variable is skipped and its default is used. The template can still reference it —
it just won't be prompted.

**Architectural fit:**
- `dex-core/src/template/variables.rs`: add `when: Option<String>` to `VariableSpec`
- Evaluation: after each variable is resolved, re-evaluate `when` conditions for
  pending variables and remove any that are now false
- Evaluation uses the same minijinja context as template rendering

---

## 5. Non-Goals

These are things the reference tools do that dex intentionally will not do:

| Feature | Why not |
|---------|---------|
| Full pre/post hook scripts (arbitrary shell/Python) | Security risk for org-distributed templates; `on_success` in §4.1 covers 90% of the use case safely |
| Template registry / public index | dex's domain is data/ML teams, not a general public marketplace; org git repos are the right distribution unit |
| npm-style auto-publish of templates | Same reason; TOML config + git remotes is sufficient |
| No-prompt-ever mode by default | CRA proved this creates egress problems; prompts + smart defaults is the right balance |
| Variable inheritance / template composition | Adds complexity that isn't yet justified; templates can share snippets via Jinja2 includes |

---

## 6. dex's Differentiation Summary

After this analysis, dex's durable differentiators against the field are:

1. **Domain depth, not breadth.** cookiecutter and Baker are generic; CRA was React-only.
   dex owns the Databricks/ML/data project lifecycle end-to-end. That domain focus
   lets us ship opinionated defaults that actually fit the user's problem.

2. **Org governance built in.** No other tool has an equivalent to dex's
   standards + presets system. This is a platform team's superpower: push
   `standards.toml` to your team's dotfiles and every project scaffolded by any
   engineer in your org inherits your org's Python version, workspace URL, etc.

3. **AI-augmented scaffolding.** `dex agent new` is not a template fill-in; it's
   a structured interview that feeds an LLM to generate a working starting point.
   No general scaffolding tool does this. This is where dex competes with
   Cursor/Copilot Workspace on their home turf.

4. **Single binary.** cookiecutter requires Python. CRA requires Node. Baker (cargo-generate)
   requires Rust. dex ships as one static binary — `curl | sh` and done.
   In a world where data teams run dex in CI, on remote VMs, and in devcontainers,
   zero runtime dependencies is a real moat.

5. **Post-scaffold project lifecycle.** `dex run`, `dex db`, pass-throughs — dex stays
   useful after day one. Cookiecutter, CRA, and Baker all disappear after scaffold.
   dex is the tool that stays in `$PATH`.

---

## 7. Implementation Priority

| # | Feature | Value | Cost | Target |
|---|---------|-------|------|--------|
| 1 | Post-scaffold activation (`on_success`) | High | Low | v0.3 |
| 2 | Answer replay (`--save-answers` / `--answers`) | High | Low | v0.3 |
| 3 | Conditional variable visibility (`when`) | Medium | Low | v0.3 |
| 4 | Template authoring tools (`dex template validate/preview`) | Medium | Medium | v0.4 |
| 5 | Remote template version pinning | Medium | Medium | v0.4 |

---

## 8. Open Questions

1. **`on_success` sandboxing:** Should we run the activation command in a subprocess
   with a restricted environment, or trust that `uv sync` is safe to run as-is?
   Probably the latter for v0.3 given the target audience (engineers who understand
   their own project setup).

2. **Answers file location:** Auto-save to `~/.config/dex/last-answers/<template>.toml`
   by default, or require explicit `--save-answers`? Cookiecutter auto-saves which
   was well-received. Worth reconsidering.

3. **Remote template cache invalidation:** `~/.cache/dex/templates/` could grow
   unboundedly. Should `dex template prune` be part of the same milestone as
   remote pinning?

4. **`dex template` namespace:** Do we want `dex template validate` and
   `dex template preview` as subcommands, or `dex init --validate ./path`
   and `dex init --preview`? The former is cleaner separation; the latter
   avoids adding a new top-level noun.
