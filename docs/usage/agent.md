# dex agent new

Scaffold a batteries-included AI agent project. The output is
coding-assistant agnostic: it works with Claude Code, Cursor, Copilot, Claude
Desktop, or any MCP client out of the box.

## Synopsis

```
dex agent new [--sdk <anthropic|openai|baml>]
              [--dir <path>] [--no-prompt]
              [--preset <profile>] [--standards <path>]
```

`dex agent new` is a thin wrapper over `dex init` that restricts template
choice to the `agent-*` family.

## SDK choice

```bash
dex agent new --sdk anthropic   # Anthropic SDK (claude-* models)
dex agent new --sdk openai      # OpenAI SDK (gpt-* models)
dex agent new --sdk baml        # BAML typed functions (any provider)
```

Omit `--sdk` to be prompted.

## Variables

Prompts asked (defaults from `template.toml`):

| Variable | Description |
|---|---|
| `project_name` | Python-valid package name (required) |
| `description` | One-sentence agent description |
| `trigger` | `user_request` / `schedule` / `event` / `upstream_system` |
| `success_criteria` | What success looks like |
| `reads` / `writes` | Data sources / sinks |
| `autonomous` | Act without confirmation (bool) |
| `example_input` / `example_output` / `bad_output` | Seeded into evals |
| `deploy_target` | `job` / `serving_endpoint` |
| `ai_tools` | Multi-select: `claude`, `cursor`, `copilot`, `generic` (default: all) |
| `include_mcp` | Write `.mcp.json` (default: true) |

## What gets written

```
my-agent/
├── AGENTS.md                     # canonical, vendor-neutral orientation doc
├── CLAUDE.md                     # 3-line pointer at AGENTS.md
├── .mcp.json                     # registers `dex mcp serve` (if include_mcp)
├── dex.toml                      # [skills] packs = ["default", "agent-dev"]
├── databricks.yml                # DAB root config
├── pyproject.toml
├── src/my_agent/
│   ├── agent.py                  # plan → act → review loop
│   ├── prompts/
│   │   ├── system.md
│   │   ├── planning.md           # planner system prompt
│   │   └── review.md             # reviewer system prompt
│   └── tools/
│       ├── __init__.py
│       ├── planner.py            # Planner tool stub
│       └── reviewer.py           # Reviewer tool stub
├── evals/
│   ├── cases/
│   │   ├── example.json
│   │   └── guardrail.json        # negative case using bad_output
│   └── run.py
├── resources/                    # DAB job / serving endpoint
└── tests/
```

Plus (installed automatically by the `on_success` hook, gated on `ai_tools`):

- `.claude/commands/*.md`, `.claude/agents/*.md`
- `.cursor/rules/*.mdc`
- `.github/copilot-instructions.md`
- `.ai-skills/commands/*.md`, `.ai-skills/agents/*.md`

The skills are the union of the `default` and `agent-dev` packs, so you get
`/build`, `/test`, `/lint` plus `/eval`, `/trace-review`, `/prompt-tune`,
`/tool-add` and agent personas like `/planner`, `/safety-reviewer` surfaced
natively in each assistant.

## Plan → Act → Review loop

`agent.py` runs three phases and traces each artifact to MLflow:

1. **Plan** — the `Planner` tool renders `prompts/planning.md` and emits a
   structured plan (`plan.md` artifact).
2. **Act** — the primary SDK call produces a draft response using
   `prompts/system.md` + the plan as context (`draft.md` artifact).
3. **Review** — the `Reviewer` tool renders `prompts/review.md` and returns a
   verdict of `accept | revise | reject` (`review.md` artifact).

For BAML, planning and review are typed BAML functions (`b.Plan`, `b.Review`)
defined alongside the main function in `baml_src/<project>.baml`.

## Using it from a coding assistant

### Claude Code / Claude Desktop

The scaffold writes `.mcp.json` and `.claude/commands/`. Open the directory
and slash commands + the `dex` MCP server resolve automatically.

### Cursor

`.cursor/rules/*.mdc` files are picked up by Cursor. `.mcp.json` is also
honored by Cursor's MCP client.

### GitHub Copilot

`.github/copilot-instructions.md` is appended with the selected skills.

### Any MCP client

Point your client at the project's `.mcp.json`, or invoke
`dex mcp serve` directly and call `scaffold_agent`, `scaffold_project`,
`get_template_variables`, or `list_templates`.

## Scaffolding from an agent via MCP

The MCP server exposes `scaffold_agent`:

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "scaffold_agent",
    "arguments": {
      "sdk": "anthropic",
      "dir": "/path/to/new-agent",
      "variables": {
        "project_name": "demo_agent",
        "description": "summarize customer tickets"
      }
    }
  }
}
```

The MCP path installs skills via the library API directly (no shell) so the
output is identical to `dex agent new` without requiring a subshell.
