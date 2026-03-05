# PRD: AI Integration

**Status:** Skeleton implemented (v0.1) / Full tools (v0.2) / Agent scaffolding (v0.3)
**Owner:** TBD

---

## 1. Problem Statement

AI assistants (Claude Code, Codex, Gemini, GitHub Copilot) need to scaffold
Databricks/MLOps projects without a human operating the CLI interactively.

Current state: `dex init` is interactive. An AI agent cannot call it programmatically
without PTY hackery or screen scraping.

Goal: expose dex operations as MCP tools so AI agents can:
- Discover available templates
- Scaffold projects with supplied variable values
- Query template variable specs before scaffolding
- Scaffold AI agents with generated system prompts

---

## 2. MCP Server Design

### Transport

stdio (standard input/output). The server is started as a subprocess by the AI client.

```bash
dex mcp serve   # starts stdio MCP server
```

### Tool definitions

| Tool | Input | Output | Status |
|------|-------|--------|--------|
| `list_templates` | none | `[{name, description, version}]` | v0.1 (implemented) |
| `scaffold_project` | `{template, directory, variables?}` | `{files_created: []}` | v0.2 |
| `get_template_variables` | `{template}` | `[{name, type, default, required, prompt}]` | v0.2 |
| `scaffold_agent` | `{name, directory, description?, generate?}` | `{files_created: []}` | v0.3 |

### Security model

- All file writes are scoped to the `directory` parameter (no path traversal).
- The server does not accept shell commands.
- No network access except through the Databricks SDK (for future auth-aware tools).
- `scaffold_project` and `scaffold_agent` must validate `directory` is within
  the user's home directory or a project root (TBD policy).

---

## 3. Integration Targets

### Claude Code (primary)

Configure via `.mcp.json` in the repository root:

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

Claude Code will start `dex mcp serve` as a subprocess and communicate via stdio.
Tools are discovered automatically from the `list_tools` response.

**Usage pattern (Claude Code):**
```
User: "Scaffold a new ETL project called user_events_pipeline"
Claude: calls list_templates, finds dabs-etl, calls scaffold_project
```

### Codex / GitHub Copilot

Same stdio transport. Codex and Copilot support MCP via the same `.mcp.json` format.

### Gemini

Gemini supports MCP. Same configuration format.

---

## 4. `.mcp.json` Configuration

Place in repository root. Committed to version control so all contributors
get MCP tool access automatically.

```json
{
  "mcpServers": {
    "dex": {
      "command": "dex",
      "args": ["mcp", "serve"],
      "env": {}
    }
  }
}
```

For org CLIs built on dex (e.g., `acme-dex`), replace `dex` with the org CLI name:

```json
{
  "mcpServers": {
    "acme-dex": {
      "command": "acme-dex",
      "args": ["mcp", "serve"]
    }
  }
}
```

---

## 5. Implementation Phases

### v0.1 — Skeleton (current)

- [x] `dex mcp serve` command exists and starts the server
- [x] `list_templates` tool is fully implemented
- [x] `scaffold_project`, `scaffold_agent`, `get_template_variables` are stubs
      (raise `NotImplementedError` with a helpful message)
- [x] Tool schemas are complete so AI agents can discover and attempt calls

### v0.2 — Full scaffold tools

- [ ] `scaffold_project`: call `scaffold_project()` from `dex._core`
- [ ] `get_template_variables`: call `parse_template_manifest()` from `dex._core`
- [ ] Error handling: return structured errors, not raw exceptions
- [ ] Integration test: Claude Code can scaffold a default project end-to-end

### v0.3 — Agent scaffolding

- [ ] `scaffold_agent`: call `scaffold_agent()` from `dex._core`
- [ ] Option to generate system prompt via Claude API within the MCP call
- [ ] Return `system_prompt` and `claude_md` in the response
- [ ] Integration test: Claude Code can scaffold an AI agent end-to-end

---

## 6. Open Questions

1. **Authentication**: Should `scaffold_project` accept Databricks auth config
   (host, token) so it can validate catalog/schema existence before scaffolding?
   Risk: adds network dependency to a local operation.

2. **Path policy**: What is the allowed `directory` range? Home dir? CWD? Any path?
   Current answer: any path (no restriction). May need to tighten for safety.

3. **Streaming**: Should `scaffold_project` stream file creation events, or return
   a single response when done? MCP supports streaming — use for large projects.

4. **org CLI MCP support**: Should `create_cli()` automatically register an `mcp serve`
   subcommand, or do org CLI authors opt in? Current plan: automatic (same as `agent`).

5. **Template source in MCP**: Can an MCP caller specify a custom template directory,
   or only embedded templates? v0.2 scope: embedded only. v0.3: support `templates_dir`.

---

## 7. Risks

| Risk | Mitigation |
|------|-----------|
| `mcp` Python package API changes | Pin `mcp>=1.0,<2.0` once stable |
| Path traversal in `directory` param | Validate in MCP handler before calling core |
| AI agent generates wrong variable values | Return variable spec before scaffold; agents should call `get_template_variables` first |
| DABs composite mode in MCP | Block `scaffold_project` for composite templates until Phase 1 (`_run_dabs_init`) is fixed |
