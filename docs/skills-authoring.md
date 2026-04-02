# Authoring Skill Packs for dex

A **skill pack** is a named collection of AI agent skills — Claude Code slash commands
and agent personas — distributed as plain markdown files with a `skills.toml` manifest.

Skill packs let your organization define and share opinionated AI workflows for any
project type: data engineering, ML, platform ops, or domain-specific tooling.

---

## Quick start

```
my-org-skills/
  skills.toml          ← required manifest
  commands/            ← slash commands (one .md per skill)
    deploy.md
    run-pipeline.md
  agents/              ← agent personas (one .md per skill)
    platform-eng.md
    ml-reviewer.md
```

1. Create the directory structure above.
2. Write a `skills.toml` manifest.
3. Write your skill `.md` files.
4. Distribute via git repo or local path.
5. Install with `dex skills init`.

---

## The `skills.toml` manifest

```toml
[pack]
name        = "my-org"
description = "My org's AI development skills"
version     = "1.0.0"

[[skills]]
name        = "deploy"
type        = "command"   # "command" | "agent"
file        = "commands/deploy.md"
description = "Deploy to production"

[[skills]]
name        = "platform-eng"
type        = "agent"
file        = "agents/platform-eng.md"
description = "Platform engineering persona"
```

**Required fields:**

| Field         | Type   | Description                                          |
|---------------|--------|------------------------------------------------------|
| `pack.name`   | string | Unique pack identifier (used in config references)   |
| `pack.description` | string | One-line description shown in `dex skills list`  |
| `pack.version` | string | SemVer string (e.g. `"1.0.0"`)                     |
| `skills[].name` | string | Slug used as the installed filename               |
| `skills[].type` | string | `"command"` or `"agent"`                          |
| `skills[].file` | string | Path relative to the pack root                    |
| `skills[].description` | string | One-line description                     |

---

## Writing command skills (`commands/*.md`)

A **command** skill tells the AI how to perform a specific action or workflow.
Write it as plain markdown — no frontmatter required.

```markdown
Deploy the application to production.

Steps:
1. Run the deploy script: `./scripts/deploy.sh --env production`
2. Verify the deployment: check the health endpoint at https://api.example.com/health
3. Notify the team in #deployments Slack channel

Before deploying:
- Confirm tests pass: `make test`
- Confirm the release tag is correct: `git tag`

Rollback:
- `./scripts/rollback.sh --env production`
```

**Tips:**
- Be specific about commands, flags, and expected outputs.
- Include failure modes and troubleshooting steps.
- Reference environment-specific config where relevant.

---

## Writing agent persona skills (`agents/*.md`)

An **agent** skill defines an AI persona — an expert identity with specific focus areas,
heuristics, and communication style.

```markdown
You are a Platform Engineer at Acme Corp.

Focus: Kubernetes cluster configuration, Terraform infrastructure, and observability.

Your expertise covers:
- EKS cluster tuning: node groups, autoscaling, spot instance strategies
- Terraform: module structure, state management, import workflows
- Observability: Prometheus/Grafana dashboards, alerting rules

When reviewing infrastructure code:
- Flag hardcoded resource limits — point to the cluster autoscaler config
- Suggest resource requests and limits for all containers
- Call out missing liveness/readiness probes
- Flag Terraform resources that drift from the org's module standards

When writing code:
- Use the org's approved Terraform module registry (registry.acme.corp)
- All EKS node groups must use spot instances with on-demand fallback
- Tag all resources with `Environment`, `Team`, and `CostCenter` tags
```

**Tips:**
- Give the agent a concrete identity, not just a job title.
- List specific heuristics and anti-patterns it should catch.
- Include org-specific context (tool names, registry URLs, conventions).
- Add examples of good and bad patterns where helpful.

---

## Install targets

Skills are installed in source-of-truth format (plain markdown) and converted to
each tool's native format during installation:

| Target    | Command                        | Agent                             |
|-----------|--------------------------------|-----------------------------------|
| `claude`  | `.claude/commands/<name>.md`   | `.claude/agents/<name>.md`        |
| `cursor`  | `.cursor/rules/<name>.mdc`     | `.cursor/rules/<name>.mdc`        |
| `copilot` | `.github/copilot-instructions.md` (appended section) | same |
| `generic` | `.ai-skills/commands/<name>.md` | `.ai-skills/agents/<name>.md`    |

---

## Distributing your pack

### Option 1: Git repository

Commit your pack to a git repo with the above structure at the root:

```
my-org-dex-skills/       ← repo root
  my-org/                ← pack directory (matches pack.name)
    skills.toml
    commands/
    agents/
```

Users register it in `~/.config/dex/config.toml`:

```toml
[[skills.remotes]]
name = "my-org"
url  = "https://github.com/my-org/dex-skills.git"
ref  = "main"
```

Or use `dex skills add`:

```bash
dex skills add https://github.com/my-org/dex-skills.git --name my-org
```

### Option 2: Local directory

Point dex at a local directory containing one or more pack directories:

```toml
# ~/.config/dex/config.toml
[skills]
dir = "~/my-org-skills"
```

### Option 3: Bundle with a template

Add a `[skills]` section to your `template.toml` to suggest packs when users
scaffold from your template:

```toml
# template.toml
[template]
name = "my-org-service"
description = "My org service template"
version = "0.1.0"

[skills]
packs = ["my-org", "default"]
```

After `dex init`, users see:
```
tip: Suggested skill packs: my-org, default
     Run `dex skills init` to install them.
```

---

## Versioning and pinning

Use semantic versioning in `skills.toml`. Pin a `ref` in config to lock:

```toml
[[skills.remotes]]
name = "my-org"
url  = "https://github.com/my-org/dex-skills.git"
ref  = "v2.1.0"   # pin to a tag
```

Update to latest:

```bash
dex skills sync --update
```

---

## Multiple packs in one repo

A single git repo can contain multiple packs as sibling directories:

```
dex-skills/             ← repo root
  data-platform/        ← pack 1
    skills.toml
    commands/
    agents/
  ml-ops/               ← pack 2
    skills.toml
    commands/
    agents/
```

dex scans the root directory and discovers all packs with a `skills.toml`.

---

## dex.toml project configuration

Record installed packs in `dex.toml` for reproducibility:

```toml
[skills]
packs   = ["default", "my-org"]
targets = ["claude", "cursor"]
```

Run `dex skills sync` to re-install based on this config. Useful in onboarding
scripts and CI environments where you want all developers to have the same skills.

---

## Reference: built-in packs

dex ships two built-in packs:

**`default`** — General-purpose AI development skills:
- Commands: `build`, `test`, `lint`, `review-pr`, `commit`
- Agents: `architect`, `code-reviewer`, `common-sense`

**`databricks`** — Databricks workflow skills:
- Commands: `deploy-bundle`, `run-job`
- Agents: `data-engineer`, `platform-engineer`
