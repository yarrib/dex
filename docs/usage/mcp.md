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

Create `.mcp.json` at your project root (or `~/.claude/mcp.json` for global config):

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

Claude Code will start the server automatically when you open the project.

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

Claude will call the appropriate tool and report the created files.

## See also

- [Installation](../installation.md)
- [dex init](init.md) — scaffold directly from the CLI
