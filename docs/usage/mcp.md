# dex mcp serve

Start the dex MCP server to expose dex tools to Claude and other MCP clients.

## Synopsis

```
dex mcp serve
```

## Overview

The MCP (Model Context Protocol) server lets AI tools like Claude Desktop and Claude Code call dex
operations directly — scaffolding projects and listing templates — without leaving the chat
interface.

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

## Wiring into Claude Desktop

Add to `claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "dex": {
      "command": "dex",
      "args": ["mcp", "serve"]
    }
  }
}
```

Restart Claude Desktop. The dex tools will appear in the tool picker.

## Wiring into Claude Code

The quickest path is the `claude mcp add` command:

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
    "dex": {
      "command": "dex",
      "args": ["mcp", "serve"]
    }
  }
}
```

Claude Code starts the server automatically when you open the project. Run `/mcp`
inside Claude Code to confirm the `dex` server is connected.

## Wiring into Cursor

Add the same block to `~/.cursor/mcp.json` (global) or `.cursor/mcp.json` (project root):

```json
{
  "mcpServers": {
    "dex": {
      "command": "dex",
      "args": ["mcp", "serve"]
    }
  }
}
```

> **PATH note:** GUI clients (Claude Desktop, Cursor) may not inherit your shell's
> `PATH`. If the server fails to start, replace `"dex"` with the absolute path to the
> binary — e.g. `"command": "/Users/you/.local/bin/dex"` (find it with `which dex`).

## Usage examples

Once connected, you can prompt Claude naturally:

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

Claude will call the appropriate tool and report the created files.

## See also

- [Installation](../installation.md)
- [dex init](init.md) — scaffold directly from the CLI
