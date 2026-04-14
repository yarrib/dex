# PRD: Org-Validated Skills & MCP Server Catalog

**Status:** Planning
**Priority:** High
**Owner:** TBD

---

## 1. Problem Statement

Teams using dex inside an organization want to distribute:
- **Skills packs** — slash commands and agent personas for Claude Code, Cursor, and Copilot
- **MCP servers** — tool integrations that AI agents can call (Databricks, internal APIs, etc.)

Today, both of these can be fetched from arbitrary git URLs configured manually in
`~/.config/dex/config.toml`. That works for individual power users but breaks down
organizationally:

1. **No approval signal.** Any git URL can be added. There is no way for a platform team
   to say "these are the vetted tools; use only these."
2. **No discoverability.** Users must already know the URL. There is no `dex skills search`
   or `dex mcp list` that shows what the org has approved.
3. **No MCP lifecycle management.** `.mcp.json` is hand-edited. There is no `dex mcp add`
   that installs a server from a vetted source, no `dex mcp remove`, no `dex mcp sync`.
4. **AI agents cannot self-configure.** Claude Code running `dex mcp serve` can list
   templates and scaffold projects, but it cannot discover or install org-approved skills
   or MCP servers on behalf of the user.

**Goal:** Introduce a *catalog* primitive — an org-internal GitHub repo that acts as the
source of truth for approved skills packs and MCP servers — and wire it through both the
CLI and the MCP tool surface so users and AI agents can consume it.

---

## 2. Goals

- Platform teams publish one catalog repo; developers and AI agents pull from it.
- `dex skills add <name>` resolves `<name>` against the registered catalog instead of
  requiring a raw git URL.
- `dex mcp add <name>` installs an org-approved MCP server and writes `.mcp.json`.
- New MCP tools (`list_catalog_skills`, `install_skill`, `list_catalog_mcp_servers`,
  `install_mcp_server`) let Claude and other agents self-configure without human
  intervention.
- Trust is explicit: catalogs are registered in user/org config, never auto-discovered
  from `dex.toml` (preventing supply-chain attacks via compromised project files).

---

## 3. Non-Goals

- A public, centrally-hosted dex catalog (out of scope; each org runs its own).
- Automatic catalog synchronization on every dex command (on-demand fetch with a
  `--refresh` flag is sufficient).
- Code signing or cryptographic provenance of catalog entries (trust is established
  by the catalog repo URL being in config, same as any git remote today).
- MCP server installation beyond writing `.mcp.json` (dex does not `npm install`,
  `pip install`, or manage runtimes; the catalog entry specifies the command and the
  user is responsible for having it available).

---

## 4. Core Concept: The Catalog

A **catalog** is a git repository (typically org-internal on GitHub) containing a
`catalog.toml` manifest that lists approved skills packs and MCP server definitions.

```
my-org-dex-catalog/
├── catalog.toml          # manifest (required)
├── skills/               # optional: inline skill pack directories
│   └── my-org-default/
│       ├── skills.toml
│       └── commands/
│           └── deploy.md
└── mcp/                  # optional: supplemental MCP config fragments
    └── databricks.json
```

### 4.1. `catalog.toml` Schema

```toml
[catalog]
name = "my-org"
description = "My Org's validated AI tooling"
version = "1.0.0"
min_dex_version = "0.4.0"

# --- Skills packs ---

[[skills]]
name = "my-org-default"
description = "Standard AI development skills for all engineers"
# Option A: remote pack (separate git repo)
url = "https://github.com/my-org/skills-default.git"
ref = "v1.2.0"

[[skills]]
name = "my-org-databricks"
description = "Databricks-specific workflows and agent personas"
url = "https://github.com/my-org/skills-databricks.git"
ref = "main"

[[skills]]
name = "my-org-platform"
description = "Platform engineering skills (infra, CI, secrets)"
# Option B: inline pack (subdirectory inside this catalog repo)
path = "skills/my-org-platform"

# --- MCP servers ---

[[mcp_servers]]
name = "databricks"
description = "Databricks MCP server — workspace, compute, SQL"
# How the AI client starts this server:
command = "uvx"
args = ["databricks-mcp-server@latest"]
# Environment variables the server needs (names only; values from user env):
env = ["DATABRICKS_HOST", "DATABRICKS_TOKEN"]
# Optional: extra Claude Code MCP config fields:
[mcp_servers.extra]
timeout = 30

[[mcp_servers]]
name = "internal-data-catalog"
description = "Internal data catalog search and lineage tools"
command = "dex-mcp-data-catalog"
args = ["--config", "${HOME}/.config/my-org/data-catalog.toml"]
env = ["CATALOG_API_KEY"]

[[mcp_servers]]
name = "secrets"
description = "HashiCorp Vault secrets access"
command = "uvx"
args = ["vault-mcp-server"]
env = ["VAULT_ADDR", "VAULT_TOKEN"]
```

### 4.2. Catalog Registration

Catalogs are registered in user config at `~/.config/dex/config.toml`:

```toml
[[catalogs]]
name = "my-org"
url = "https://github.com/my-org/dex-catalog.git"
ref = "main"          # branch, tag, or commit SHA

# Multiple catalogs are supported (resolved in order; first match wins):
[[catalogs]]
name = "community"
url = "https://github.com/dex-community/catalog.git"
ref = "stable"
```

Catalogs are fetched to `~/.cache/dex/catalogs/<name>/` using the same remote-fetch
mechanism already used for templates and skills.

**Why user config, not `dex.toml`?**
`dex.toml` is project-scoped and checked in. A malicious or compromised `dex.toml`
could point `[catalogs]` at an attacker-controlled repo. Placing catalog registration
in `~/.config/dex/config.toml` means it requires explicit user/admin action on the
machine, equivalent to trusting a remote.

For org-wide rollout, platform teams can provision `~/.config/dex/config.toml` via
MDM, dotfiles, or a bootstrap script — the same pattern used today for org `dex.toml`
templates.

---

## 5. CLI Interface

### 5.1. Catalog Management

```
dex catalog add <url> [--name <name>] [--ref <ref>]
    Register a catalog. Fetches immediately to validate.
    --name defaults to the catalog's [catalog.name] field.
    --ref defaults to "main".

dex catalog list
    List registered catalogs with name, URL, ref, and last-fetched time.

dex catalog refresh [<name>]
    Re-fetch catalogs from their remotes. Refreshes all if <name> omitted.

dex catalog remove <name>
    Unregister a catalog and remove its cache.
```

### 5.2. Skills — Catalog-Aware Changes

Existing commands gain catalog resolution. No breaking changes.

```
dex skills list [--catalog <name>]
    List installed skill packs AND available packs from registered catalogs.
    Shows: name, source (installed/catalog/<catalog-name>), description.

dex skills add <pack-name> [--catalog <name>] [--target <claude|cursor|copilot>]
    Install a skill pack by name. Resolves against registered catalogs.
    Falls back to already-configured remote URLs for backwards compatibility.
    --catalog scopes resolution to a specific catalog.
    --target selects install targets (defaults to dex.toml [skills] targets).

dex skills sync
    (existing) Re-install all packs from their sources. No change needed —
    installed packs already record their source URL.
```

### 5.3. MCP — New Commands

```
dex mcp list [--catalog <name>]
    List MCP servers: currently installed (in .mcp.json) and available from catalogs.
    Shows: name, status (installed/available), description, command.

dex mcp add <name> [--catalog <name>] [--scope <project|user>]
    Install an MCP server from the catalog into .mcp.json (--scope project, default)
    or into ~/.claude/claude_desktop_config.json (--scope user).
    Prints the env vars the server needs (values must be set by the user).

dex mcp remove <name> [--scope <project|user>]
    Remove a server entry from .mcp.json or user config.

dex mcp sync [--catalog <name>]
    Re-validate all installed MCP servers against their catalog entries.
    Warns if a server's command or args have changed upstream.
    Does not auto-update (user must run `dex mcp add` again to update).
```

#### Example: `dex mcp add databricks`

```
$ dex mcp add databricks

Installing MCP server: databricks
  Source: my-org catalog (github.com/my-org/dex-catalog @ main)
  Command: uvx databricks-mcp-server@latest

Updated .mcp.json.

Required environment variables (set these before starting your AI client):
  DATABRICKS_HOST   - Databricks workspace URL
  DATABRICKS_TOKEN  - Personal access token or OAuth token
```

Generated `.mcp.json` entry:

```json
{
  "mcpServers": {
    "dex": {
      "command": "dex",
      "args": ["mcp", "serve"]
    },
    "databricks": {
      "command": "uvx",
      "args": ["databricks-mcp-server@latest"],
      "env": {
        "DATABRICKS_HOST": "${DATABRICKS_HOST}",
        "DATABRICKS_TOKEN": "${DATABRICKS_TOKEN}"
      }
    }
  }
}
```

---

## 6. MCP Tool Surface (Programmatic Access)

New tools added to `dex mcp serve` so AI agents (Claude Code, Codex, Gemini) can
discover and install catalog items without human CLI intervention.

| Tool | Input | Output | Phase |
|------|-------|--------|-------|
| `list_catalog_skills` | `{catalog?: string}` | `[{name, description, catalog, installed}]` | v0.5 |
| `install_skill` | `{name, catalog?: string, targets?: string[]}` | `{installed_to: []}` | v0.5 |
| `list_catalog_mcp_servers` | `{catalog?: string}` | `[{name, description, command, env_vars, installed}]` | v0.5 |
| `install_mcp_server` | `{name, catalog?: string, scope?: "project"\|"user"}` | `{mcp_json_path, env_vars_needed: []}` | v0.5 |
| `refresh_catalog` | `{name?: string}` | `{refreshed: []}` | v0.5 |

### Tool Detail: `install_mcp_server`

**Input:**
```json
{
  "name": "databricks",
  "catalog": "my-org",
  "scope": "project"
}
```

**Output:**
```json
{
  "mcp_json_path": "/workspace/my-project/.mcp.json",
  "server_name": "databricks",
  "command": "uvx",
  "args": ["databricks-mcp-server@latest"],
  "env_vars_needed": [
    { "name": "DATABRICKS_HOST", "description": "Databricks workspace URL" },
    { "name": "DATABRICKS_TOKEN", "description": "Personal access token or OAuth token" }
  ]
}
```

**Security note:** `install_mcp_server` writes only to `.mcp.json` or the user's
Claude Desktop config. It never executes the MCP server command itself and never
reads or sets environment variable values.

### Usage Pattern (Claude Code)

```
User: "Set up the Databricks MCP server for this project"

Claude:
1. calls list_catalog_mcp_servers → finds "databricks" in my-org catalog
2. calls install_mcp_server {name: "databricks", scope: "project"}
3. reports back: "I've added the Databricks MCP server to .mcp.json.
   You need to set DATABRICKS_HOST and DATABRICKS_TOKEN in your environment
   before restarting Claude Code."
```

```
User: "Install the standard org skills"

Claude:
1. calls list_catalog_skills → finds "my-org-default" and "my-org-databricks"
2. calls install_skill {name: "my-org-default"}
3. calls install_skill {name: "my-org-databricks"}
4. reports: "Installed 2 skill packs: my-org-default (8 skills) and
   my-org-databricks (4 skills) for Claude Code."
```

---

## 7. Data Model Changes

### 7.1. User Config (`~/.config/dex/config.toml`)

```toml
# New top-level section:
[[catalogs]]
name = "my-org"
url = "https://github.com/my-org/dex-catalog.git"
ref = "main"
```

### 7.2. Cached Catalog State (`~/.cache/dex/catalogs/`)

```
~/.cache/dex/catalogs/
└── my-org/
    ├── catalog.toml        # fetched manifest
    ├── skills/             # inline skill packs (if any)
    └── .fetch-meta.toml    # last-fetched timestamp, resolved commit SHA
```

`.fetch-meta.toml`:
```toml
fetched_at = "2026-04-14T12:00:00Z"
resolved_sha = "abc123def456"
```

### 7.3. `dex-core` Types (new)

```rust
pub struct CatalogRemote {
    pub name: String,
    pub url: String,
    pub git_ref: String,
}

pub struct CatalogManifest {
    pub catalog: CatalogMeta,
    pub skills: Vec<CatalogSkillEntry>,
    pub mcp_servers: Vec<CatalogMcpEntry>,
}

pub struct CatalogSkillEntry {
    pub name: String,
    pub description: String,
    pub url: Option<String>,    // remote pack
    pub git_ref: Option<String>,
    pub path: Option<String>,   // inline pack (relative to catalog root)
}

pub struct CatalogMcpEntry {
    pub name: String,
    pub description: String,
    pub command: String,
    pub args: Vec<String>,
    pub env: Vec<EnvVarSpec>,
    pub extra: Option<toml::Value>,
}

pub struct EnvVarSpec {
    pub name: String,
    pub description: Option<String>,
}
```

---

## 8. Implementation Plan

### Phase 1 — Catalog Fetch & Skills Resolution (v0.5)

**dex-core:**
- [ ] `CatalogManifest`, `CatalogSkillEntry`, `CatalogMcpEntry`, `EnvVarSpec` types
- [ ] `UserConfig`: add `catalogs: Vec<CatalogRemote>` field
- [ ] `catalog::fetch(remote: &CatalogRemote, cache_dir: &Path) -> Result<CatalogManifest>`
  — reuse existing git fetch infrastructure from `template::remote`
- [ ] `catalog::load_cached(name: &str, cache_dir: &Path) -> Result<CatalogManifest>`
- [ ] `catalog::resolve_skill(name: &str, catalogs: &[CatalogManifest]) -> Result<SkillSource>`
  — returns either a `RemoteSource` or a local cache path for inline packs
- [ ] Update `skills::install` to accept `SkillSource` (already handles remote URLs;
  inline paths are just local directories, same as existing path resolution)

**dex-cli:**
- [ ] `commands/catalog.rs` — `dex catalog add/list/refresh/remove`
- [ ] Register `CatalogCommand` in `main.rs`
- [ ] Update `commands/skills.rs`: `dex skills add <name>` resolves via catalog first,
  falls back to raw URL (backwards compatibility)
- [ ] Update `commands/skills.rs`: `dex skills list` shows catalog-available packs

**Tests:**
- [ ] Unit test: `catalog::resolve_skill` finds correct entry across two catalogs
- [ ] Unit test: name collision (same name in two catalogs) → first-registered wins
- [ ] Integration test: `dex catalog add` + `dex skills add <name>` end-to-end with
  a fixture catalog repo

### Phase 2 — MCP Lifecycle Commands (v0.5)

**dex-core:**
- [ ] `mcp_config::read_mcp_json(dir: &Path) -> Result<McpConfig>` — parse `.mcp.json`
- [ ] `mcp_config::write_mcp_json(dir: &Path, config: &McpConfig) -> Result<()>`
- [ ] `mcp_config::add_server(config: &mut McpConfig, entry: &CatalogMcpEntry) -> Result<()>`
  — idempotent; warns if server already present with different config
- [ ] `mcp_config::remove_server(config: &mut McpConfig, name: &str) -> Result<()>`
- [ ] `catalog::resolve_mcp_server(name: &str, catalogs: &[CatalogManifest]) -> Result<&CatalogMcpEntry>`

**dex-cli:**
- [ ] `commands/mcp.rs`: add `list`, `add`, `remove`, `sync` subcommands
- [ ] `dex mcp add` — calls `catalog::resolve_mcp_server`, calls `mcp_config::add_server`,
  prints required env vars
- [ ] `dex mcp list` — shows installed (from `.mcp.json`) + available (from catalogs)
- [ ] `dex mcp remove` — calls `mcp_config::remove_server`
- [ ] `dex mcp sync` — diffs installed vs. catalog entries, warns on drift

**Tests:**
- [ ] Unit test: `mcp_config::add_server` produces correct `.mcp.json` for a fixture entry
- [ ] Unit test: idempotent add (re-adding same server is a no-op)
- [ ] Integration test: `dex mcp add databricks` with fixture catalog

### Phase 3 — MCP Tool Surface (v0.5)

**dex-cli (mcp server handler):**
- [ ] `list_catalog_skills` tool — calls `catalog::load_cached` for all registered catalogs,
  returns skill entries with `installed` flag
- [ ] `install_skill` tool — calls `catalog::resolve_skill` + `skills::install`
- [ ] `list_catalog_mcp_servers` tool — same pattern as `list_catalog_skills`
- [ ] `install_mcp_server` tool — calls `catalog::resolve_mcp_server` + `mcp_config::add_server`
- [ ] `refresh_catalog` tool — calls `catalog::fetch` for named or all catalogs

**Tests:**
- [ ] Integration test: MCP server responds to `list_catalog_skills` with fixture catalog
- [ ] Integration test: `install_mcp_server` writes correct `.mcp.json`

---

## 9. Security Model

| Concern | Mitigation |
|---------|-----------|
| Untrusted catalog URL in `dex.toml` | Catalogs are registered only in `~/.config/dex/config.toml`, never read from project `dex.toml` |
| Catalog repo compromise (dependency confusion) | Org pins catalog to a specific commit SHA via `ref`; `dex catalog refresh` shows the new SHA before updating |
| MCP server command injection via catalog entry | `command` and `args` are used with `std::process::Command` (already the pass-through pattern); no shell interpolation |
| Env var values exposed via MCP tool output | `install_mcp_server` writes `${ENV_VAR}` placeholder strings to `.mcp.json`, never reads or returns actual values |
| Path traversal in inline skill `path` | `path` must be relative; validated to not contain `..` or absolute prefix before resolving against cache root |
| `.mcp.json` written outside project root | `install_mcp_server` with `scope: "project"` validates target path is within CWD; user-scope writes only to Claude Desktop config path |

---

## 10. Open Questions

1. **Catalog pinning UX.** Should `dex catalog add` default to pinning the resolved
   SHA on first fetch (like a lockfile), or always track the branch tip? Branch tip is
   simpler; SHA pinning is safer. Proposal: default to branch, add `--pin` flag.

2. **Org-wide config distribution.** Platform teams need a way to push catalog
   registration to developer machines without each person running `dex catalog add`.
   Options: (a) bootstrap script that writes `~/.config/dex/config.toml`, (b) a
   future `dex org init` command, (c) document the MDM/dotfiles pattern. No new
   dex feature needed for v0.5; document (a) and (c).

3. **Catalog `dex.toml` integration.** Should `dex init` from a catalog template
   automatically register the catalog that shipped the template? Convenient but blurs
   the project/user config boundary. Proposal: no auto-registration; the template's
   README tells developers to run `dex catalog add`.

4. **Catalog versioning & breaking changes.** If a catalog removes an MCP server entry,
   `dex mcp sync` should warn that a server is installed but no longer in the catalog
   rather than silently removing it. What's the right UX for "orphaned" entries?

5. **Inline skills pack resolution via MCP tool.** `install_skill` for an inline pack
   (path-based, inside the catalog cache) works differently from a remote URL. Should
   the MCP tool copy the files from cache to `~/.cache/dex/skills/` so that `dex skills
   sync` can re-fetch them? Or treat inline packs as a special immutable source?
   Proposal: copy to a stable cache path at install time, record the catalog name +
   catalog ref as the "source" in `dex.toml [skills]`.

6. **Multi-catalog conflict resolution.** When two catalogs define a skill with the
   same name, first-registered-wins is simple but opaque. Should `dex skills list`
   show which catalog would win for each name, and should `dex skills add my-skill
   --catalog foo` always allow disambiguation? Proposal: yes to both.

---

## 11. Risks

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|-----------|
| Git fetch latency degrades CLI startup | Medium | Medium | Fetch is on-demand (`dex catalog refresh`), never on every command; cached manifest is read from disk |
| Org catalog abandoned / URL goes stale | Low | High | `dex catalog refresh` fails gracefully; cached manifest is used with a staleness warning |
| `.mcp.json` merge conflicts | Medium | Low | `dex mcp sync` warns on drift; users resolve manually; `.mcp.json` is typically machine-generated |
| `install_mcp_server` MCP tool used to write outside project | Low | High | Path validation in dex-core before write; covered by security model above |
| Catalog schema evolves incompatibly | Low | Medium | `min_dex_version` in `[catalog]`; dex errors with a clear upgrade message if catalog requires newer version |
