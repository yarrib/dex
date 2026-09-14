# dex agent-env init

Generate a reproducible, non-interactive dev environment for coding agents
from a `dex.agent-env.toml` manifest.

## Synopsis

```
dex agent-env init [--dir <path>] [--dry-run]
```

## Overview

Running coding agents (Claude Code, etc.) against Databricks + Azure projects
usually means manual, per-project setup: interactive auth, ad-hoc devcontainers,
no standard verify step. `dex agent-env` turns that setup into scaffolded,
versioned, reviewable config — a project declares what it needs once, in
`dex.agent-env.toml` at the project root, and `dex agent-env init` generates:

- `.devcontainer/devcontainer.json` + `.devcontainer/Dockerfile` — a pinned
  base image, any extra apt packages, and a `postCreateCommand` that clones
  the project's other repos and runs the auth bootstrap script.
- `.devcontainer/auth-bootstrap.sh` — authenticates to Databricks via
  service-principal OAuth M2M and writes a scoped `.databrickscfg` profile,
  with zero interactive steps.
- `.devcontainer/verify.sh` — runs the manifest's `[[verify.steps]]` in
  order, with a clean pass/fail exit code.
- `.github/workflows/agent-env.yml` — rebuilds the devcontainer base image on
  the configured schedule and runs `verify.sh` on pull requests.

`init` is **idempotent**: it regenerates every file from the manifest on each
run — it never hand-patches an existing file — and reports each as created,
updated, or unchanged. Re-run it any time you edit the manifest.

This composes with (doesn't replace) `databricks.yml` / Databricks Asset
Bundles — `dex agent-env` only manages the dev-environment and verify layer.

## Manifest schema (`dex.agent-env.toml`)

| Section | Field | Required | Description |
|---|---|---|---|
| (root) | `version` | yes | Manifest schema version. Only `1` is supported. |
| `[base]` | `image` | yes | Devcontainer base image, e.g. `mcr.microsoft.com/devcontainers/python:3.12`. |
| `[base]` | `extra_packages` | no | Package names layered onto the base image via `apt-get install` (v1 treats every entry as an apt target — devcontainer base images are Debian-based). |
| `[base]` | `repos` | no | Repos to clone in `postCreateCommand`, as `org/repo` slugs. Defaults to `["self"]`. `"self"` is skipped — it's already mounted by the devcontainer. |
| `[auth]` | `provider` | yes | Only `"databricks-oauth-m2m"` is supported in v1. |
| `[auth]` | `workspace` | yes | A short label (`^[A-Za-z0-9_-]+$`). Becomes the `.databrickscfg` profile name. |
| `[auth]` | `workspace_host` | yes | The literal `https://...` Databricks workspace URL used for the OAuth token exchange. |
| `[auth.service_principal_env]` | `client_id` | yes | **Name** of the environment variable holding the service-principal client ID (not the value). |
| `[auth.service_principal_env]` | `client_secret` | yes | **Name** of the environment variable holding the service-principal client secret. |
| `[auth.scope]` | `catalog` | yes | Sandbox catalog the service principal is scoped to. |
| `[auth.scope]` | `schema` | yes | Sandbox schema the service principal is scoped to. |
| `[[verify.steps]]` | `name` | yes | Step label (unique within the manifest). |
| `[[verify.steps]]` | `run` | yes | Shell command for the step. |
| `[[verify.steps]]` | `always` | no | If `true`, the step still runs even if an earlier non-`always` step failed (default `false`). Use this for teardown, e.g. `databricks bundle destroy`. |
| `[refresh]` | `schedule` | no | `"nightly"`, `"weekly"`, or a raw 5-field cron expression — controls when the generated CI workflow rebuilds the base image. |

## Worked example

```toml
# dex.agent-env.toml
version = 1

[base]
image = "mcr.microsoft.com/devcontainers/python:3.12"
extra_packages = ["ripgrep", "jq"]
repos = ["self", "my-org/shared-libs"]

[auth]
provider = "databricks-oauth-m2m"
workspace = "dev"
workspace_host = "https://adb-123.4.azuredatabricks.net"

[auth.service_principal_env]
client_id = "DATABRICKS_CLIENT_ID"
client_secret = "DATABRICKS_CLIENT_SECRET"

[auth.scope]
catalog = "main"
schema = "agent_dev"

[[verify.steps]]
name = "lint"
run = "ruff check ."

[[verify.steps]]
name = "test"
run = "pytest -q"

[[verify.steps]]
name = "cleanup"
run = "rm -rf /tmp/agent-scratch"
always = true

[refresh]
schedule = "nightly"
```

```bash
# Preview what would be written
dex agent-env init --dry-run

# Generate the environment
dex agent-env init
```

## Generated structure

```
my-project/
├── dex.agent-env.toml           # manifest (hand-authored)
├── .devcontainer/
│   ├── devcontainer.json        # pinned image, repo clones, env forwarding
│   ├── Dockerfile                # base image + apt packages
│   ├── auth-bootstrap.sh         # Databricks OAuth M2M -> .databrickscfg
│   └── verify.sh                 # runs [[verify.steps]] in order
└── .github/workflows/
    └── agent-env.yml             # rebuilds base image on schedule, runs verify on PRs
```

## Auth bootstrap and `.databrickscfg`

`auth-bootstrap.sh` reads the credentials from the environment variables
*named* in `[auth.service_principal_env]` (never hardcoded values), performs
the Databricks OAuth client-credentials exchange against `workspace_host`,
and writes a `.databrickscfg` profile named exactly `[auth].workspace` with
`auth_type = oauth-m2m` — so the Databricks CLI itself refreshes tokens
rather than a short-lived access token going stale mid-session. It fails
loudly (non-zero exit, clear stderr message) if either env var is unset or
the token exchange fails; it never silently no-ops. The secret does land in
plaintext in `~/.databrickscfg` (mode `600`) — that's inherent to how the
Databricks CLI consumes `oauth-m2m` profiles, not a dex shortcut.

**Security note:** scope the service principal to a sandbox catalog/schema
(`[auth.scope]`), not the full workspace — least privilege. The devcontainer's
`remoteEnv` forwards the two credential env vars from the host into the
container; in CI, configure them as repository secrets with the same names.

## Verify step semantics

Steps run in declared order. A non-`always` step's failure halts the
remaining non-`always` steps but `always: true` steps still run regardless
(the standard use case is teardown, e.g. `databricks bundle destroy`, that
must run even if an earlier step failed). The script's exit code is `0` only
if every non-`always` step succeeded.

**Trust boundary:** `verify.steps[].run` is interpolated as raw shell with no
escaping — the same trust boundary that already exists for `dex.toml`'s
`[tasks.*].command`. dex trusts the project's own manifest; it isn't a
sandbox for untrusted input.

## Idempotency

Re-running `dex agent-env init` after editing the manifest regenerates every
generated file from scratch and reports each as `created`, `updated`, or
`unchanged` — never a manual patch, never drift. `--dry-run` shows the same
summary without writing anything.
