# dex mcp serve

Start the dex MCP server to expose dex tools to Claude and other MCP clients.

## Synopsis

```
dex mcp serve
```

## Overview

The MCP (Model Context Protocol) server lets AI coding tools — Claude Code, Claude
Desktop, Cursor, VS Code / GitHub Copilot, OpenAI Codex, Zed, Google Antigravity,
and any other MCP-capable client — call dex operations directly: scaffolding
projects, listing templates, and creating agents without leaving the chat interface.

## Available tools

| Tool | Description |
|---|---|
| `list_templates` | Returns all built-in templates with names and descriptions |
| `get_template_variables` | Returns variable specs for a named template |
| `scaffold_project` | Scaffolds a project from a template into a directory |
| `scaffold_agent` | Scaffolds a batteries-included AI agent project (`sdk` = `anthropic`, `openai`, or `baml`); also installs the `default` + `agent-dev` skill packs |

## Installation

Install the `dex` binary first — the MCP server is built in, no separate install needed.

**Install script (Linux/macOS):**

```bash
curl -sSf https://raw.githubusercontent.com/yarrib/dex/main/install.sh | sh
```

**Build from source:**

```bash
cargo install --path crates/dex-cli
```

See [Installation](../installation.md) for full details.

## Wiring into a client

Every client launches the same command — `dex mcp serve` — over stdio. Only the
config file location and its surrounding shape differ. The shapes you'll see are:

- **`mcpServers` JSON** — Claude Desktop, Claude Code, Cursor, Antigravity
- **`servers` JSON** (with `"type": "stdio"`) — VS Code / GitHub Copilot
- **`context_servers` JSON** — Zed
- **`mcp_servers` TOML** — OpenAI Codex CLI

> **PATH note (read this first):** GUI apps (Claude Desktop, Cursor, Zed,
> Antigravity, VS Code) often don't inherit your shell's `PATH`, so a bare
> `"dex"` may fail with "command not found". If a server won't start, run
> `which dex` and use the absolute path instead — e.g.
> `"command": "/Users/you/.local/bin/dex"`.

### Claude Code

Quickest path is the CLI:

```bash
# project scope — writes .mcp.json in the current directory (shareable with your team)
claude mcp add dex --scope project -- dex mcp serve

# user scope — available across all your projects
claude mcp add dex --scope user -- dex mcp serve
```

Or create `.mcp.json` at your project root by hand:

```json
{
  "mcpServers": {
    "dex": { "command": "dex", "args": ["mcp", "serve"] }
  }
}
```

Claude Code starts the server automatically when you open the project. Run `/mcp`
to confirm the `dex` server is connected.

### Claude Desktop

Add to `claude_desktop_config.json` (Settings → Developer → Edit Config), then
restart the app. The dex tools appear in the tool picker.

```json
{
  "mcpServers": {
    "dex": { "command": "dex", "args": ["mcp", "serve"] }
  }
}
```

### Cursor

Add to `~/.cursor/mcp.json` (global) or `.cursor/mcp.json` (project root):

```json
{
  "mcpServers": {
    "dex": { "command": "dex", "args": ["mcp", "serve"] }
  }
}
```

### VS Code / GitHub Copilot

VS Code (agent mode / Copilot) uses a `servers` key and an explicit
`"type": "stdio"`. Add `.vscode/mcp.json` to your workspace (or run the
**MCP: Add Server** command):

```json
{
  "servers": {
    "dex": { "type": "stdio", "command": "dex", "args": ["mcp", "serve"] }
  }
}
```

Open the Copilot Chat **Agent** view and check the tools picker; dex's tools
appear once the server starts. (The standalone GitHub Copilot CLI uses its own
`copilot mcp add` command with the same `dex mcp serve` invocation.)

### OpenAI Codex CLI

Codex uses TOML, not JSON. Add to `~/.codex/config.toml` (or run
`codex mcp add`):

```toml
[mcp_servers.dex]
command = "dex"
args = ["mcp", "serve"]
```

### Zed

Zed calls them *context servers*. Add to `settings.json` (open with
`cmd`/`ctrl`+`,`):

```json
{
  "context_servers": {
    "dex": {
      "source": "custom",
      "command": "dex",
      "args": ["mcp", "serve"],
      "env": {}
    }
  }
}
```

Zed restarts the context server on save — no editor restart needed. Check the
Agent Panel settings for a green indicator next to `dex`.

### Google Antigravity

Antigravity uses the `mcpServers` shape at
`~/.gemini/config/mcp_config.json`. Either edit that file directly, or in the
IDE open the **MCP Servers** panel (the `...` dropdown on the agent panel) →
**Manage MCP Servers** → **View raw config**:

```json
{
  "mcpServers": {
    "dex": { "command": "dex", "args": ["mcp", "serve"] }
  }
}
```

### Any other MCP client

dex follows the standard MCP stdio transport, so any compliant client works.
Whatever the config format, the two things it needs are the **command** (`dex`)
and its **args** (`["mcp", "serve"]`). Map those onto the client's schema.

## Usage examples

Once connected, you can prompt your assistant naturally:

**List templates:**
```
What dex templates are available?
```

**Inspect a template:**
```
What variables does the dabs-package template need?
```

**Scaffold a project:**
```
Scaffold a new dabs-package project called my_pipeline in ~/projects/my_pipeline
```

**Scaffold an AI agent:**
```
Scaffold an anthropic agent called triage_bot in ~/projects/triage_bot
```

The assistant will call the appropriate tool and report the created files.

## See also

- [Installation](../installation.md)
- [dex init](init.md) — scaffold directly from the CLI
