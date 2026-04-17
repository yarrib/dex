# AGENTS.md — AI Agent Integration for dex

## What this is

`dex` supports AI coding assistants (Claude Code, Cursor, Copilot, Claude
Desktop, or any MCP client) through three surfaces:

1. **Skill packs** — vendor-neutral markdown skills installed into each tool's
   native location by `dex skills init`.
2. **MCP server** (`dex mcp serve`) — programmatic tool access for autonomous
   agents over the Model Context Protocol.
3. **Agent scaffolds** — `dex agent new` generates a batteries-included agent
   project with planner, reviewer, evals, MCP config, and `AGENTS.md`.

---

## Quick Start (any coding assistant)

Skills are authored once as plain markdown and installed into whichever tools
you use:

```bash
dex skills init              # interactive: pick packs + targets
dex skills init --yes \
    --packs default,agent-dev \
    --targets claude,cursor,copilot,generic
```

Available slash commands (surfaced in Claude Code, Cursor, etc.):

```
/build /test /lint /review-pr /commit              # default pack
/eval /trace-review /prompt-tune /tool-add         # agent-dev pack
```

Agent personas:

```
/architect /code-reviewer /common-sense            # default pack
/planner /prompt-engineer /eval-designer /safety-reviewer   # agent-dev pack
```

---

## Agent Scaffolding

```bash
dex agent new                      # prompts for SDK
dex agent new --sdk anthropic      # or: openai | baml
```

Generated output is coding-assistant agnostic. Every scaffold produces:

- `AGENTS.md` — canonical vendor-neutral orientation doc.
- `CLAUDE.md` — 3-line pointer at `AGENTS.md` (Claude Code convention).
- `.mcp.json` — registers `dex mcp serve` for any MCP-capable client.
- `src/<name>/agent.py` — plan → act → review loop with MLflow tracing.
- `src/<name>/tools/{planner,reviewer}.py` — wired into the loop.
- `src/<name>/prompts/{system,planning,review}.md` — editable.
- `evals/cases/*.json` — including a `guardrail.json` negative case.
- `.claude/`, `.cursor/rules/`, `.github/copilot-instructions.md`,
  `.ai-skills/` — populated automatically by `on_success` → `dex skills init`.

The `ai_tools` variable selects which assistant surfaces get written (default:
all four). The `include_mcp` variable toggles `.mcp.json` (default: true).

---

## MCP Server

Expose dex operations as tools over stdio:

```bash
dex mcp serve
```

`.mcp.json` registers the server in Claude Code, Claude Desktop, Cursor, and
any MCP client:

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
| `get_template_variables` | implemented | Return variable spec for a template |
| `scaffold_project` | implemented | Scaffold a project from a template |
| `scaffold_agent` | implemented | Scaffold an AI agent (sdk = anthropic/openai/baml) |

---

## Repository conventions for AI agents

- **Language**: Rust (crates/dex-core, crates/dex-cli, crates/dex-py)
- **Build**: `cargo build`; `cargo test`; `cargo clippy -- -D warnings`
- **Config**: TOML everywhere — no YAML, no JSON for config
- **Templates**: Jinja2 syntax (`.j2` extension), rendered by minijinja in Rust
- **Skills**: plain markdown under `skills/<pack>/{commands,agents}/`
- **Error messages**: user-facing and actionable — no stack traces in user output
- **UI**: `dex-cli` only — `dex-core` never touches the terminal

Full rules: see `CLAUDE.md`.
