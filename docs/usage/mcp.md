# dex mcp serve

Start the dex MCP server to expose dex tools to Claude and other MCP clients.

## Synopsis

```
dex mcp serve
```

## Overview

The MCP (Model Context Protocol) server lets AI tools like Claude Desktop call dex operations directly — scaffolding projects, listing templates, and creating agents — without leaving the chat interface.

## Available tools

| Tool | Status | Description |
|---|---|---|
| `list_templates` | Implemented | Returns all built-in templates with names and descriptions |
| `scaffold_project` | Stub | Scaffold a project from a template |
| `scaffold_agent` | Stub | Run the `dex agent new` Q&A flow |
| `get_template_variables` | Stub | Return variable specs for a template |

## Wiring into Claude Desktop

Add the server to your `claude_desktop_config.json`:

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

Add to your project's `CLAUDE.md` or run from the terminal:

```bash
dex mcp serve
```

Claude Code will auto-detect the running MCP server on stdio.

## Example: list templates via Claude

Once connected, you can prompt Claude:

```
What templates does dex have?
```

Claude will call `list_templates` and return the current built-in list.

## See also

- [AGENTS.md](https://github.com/yarrib/dex/blob/main/AGENTS.md) — full AI integration guide
- [dex agent new](agent.md) — scaffold an agent project first
