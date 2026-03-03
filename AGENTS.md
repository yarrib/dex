# AGENTS.md — AI Agent Integration for dex

## What this is

`dex` supports AI agent workflows through two surfaces:

1. **Claude Code skills** (`.claude/commands/`) — persona prompts for human+AI pair work
2. **MCP server** (`dex mcp serve`) — programmatic tool access for autonomous agents

---

## Quick Start (Claude Code)

```bash
# Run a slash command in Claude Code:
/build       # build the project
/test        # run the test suite
/lint        # run linters
/scaffold    # scaffold a new project via dex init
/agent-new   # scaffold a new agent via dex agent new

# Persona modes:
/architect       # review architecture and design
/code-writer     # implement a spec, no gold-plating
/code-reviewer   # peer review: flag issues, check conventions
/new-user        # onboarding walkthrough for a new team member
/devils-advocate # challenge decisions, prevent expensive mistakes
/common-sense    # sanity check complexity and scope
```

---

## MCP Server

The MCP server exposes dex operations as tools over stdio transport,
enabling AI agents to scaffold projects without a human CLI interaction.

### Start the server

```bash
dex mcp serve
```

### Configure in `.mcp.json` (repo root)

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

### Available tools

| Tool | Status | Description |
|------|--------|-------------|
| `list_templates` | implemented | List all available templates |
| `scaffold_project` | stub (v0.2) | Scaffold a project from a template |
| `scaffold_agent` | stub (v0.3) | Scaffold an AI agent |
| `get_template_variables` | stub (v0.2) | Get variable spec for a template |

---

## Repository conventions for AI agents

- **Language**: Rust (crates/dex-core, crates/dex-py) + Python (python/dex/)
- **Build**: `make dev` to build; `make test` to test; `make lint` to lint
- **Config**: TOML everywhere — no YAML, no JSON for config
- **Templates**: Jinja2 syntax (`.j2` extension), rendered by minijinja in Rust
- **Error messages**: user-facing and actionable — no stack traces in user output
- **FFI boundary**: dex-py is a thin bridge; all logic belongs in dex-core
- **UI**: Python layer only — dex-core never touches the terminal

Full rules: see `CLAUDE.md`.

---

## Known issues (flag for bug-fix branch)

- `AgentScaffoldResultPy` drops `system_prompt` and `claude_md` fields
- `agent_new` name/description logic has a no-op guard
- Slugifier produces hyphenated package names (invalid Python imports)
- `_run_dabs_init` is defined but never called (DABs composite templates not yet wired up)

Do not attempt to fix these inline — they are scoped to a dedicated bug-fix branch.
