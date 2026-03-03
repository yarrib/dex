# dex agent new

Scaffold an AI agent project via an interactive Q&A flow.

## Synopsis

```
dex agent new
```

## Q&A flow

`dex agent new` asks a series of questions to understand the agent's purpose, and then generates a project skeleton with a `CLAUDE.md`, `system_prompt.md`, and starter code.

**Questions asked:**

1. **Name** — short identifier for the agent
2. **Description** — what the agent does in one sentence
3. **Trigger** — how the agent is activated (`user_request`, `schedule`, `event`, `upstream_system`)
4. **Success criteria** — how you know the agent succeeded
5. **Reads** — data sources the agent reads from
6. **Writes** — data sinks or side effects
7. **Handoff** — whether the agent hands off to a human
8. **Autonomous** — whether the agent runs without human review
9. **Example input** — a concrete example of what the agent receives
10. **Example output** — what a good response looks like
11. **Bad output** — what a bad response looks like (for guardrails)
12. **Deploy target** — `job`, `serving_endpoint`, or `interactive`

## Generated files

```
<agent_name>/
├── CLAUDE.md            # Agent instructions for Claude Code
├── system_prompt.md     # System prompt template
├── main.py              # Entry point skeleton
└── README.md            # Setup and usage
```

## MCP integration

After scaffolding, the agent project can be served via the dex MCP server:

```bash
dex mcp serve
```

This exposes the agent's tools to Claude and other MCP clients.
