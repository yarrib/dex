Scaffold a new AI agent using `dex agent new`.

Usage:
```bash
dex agent new --name "My Agent" --dir /path/to/agent
```

With AI-generated system prompt and CLAUDE.md:
```bash
dex agent new --name "Table Anomaly Monitor" --dir /tmp/my-agent
# → prompts for description, then calls Claude API to generate system prompt
```

Without AI generation (dry run):
```bash
dex agent new --name "My Agent" --dir /tmp/my-agent --no-generate
```

What gets generated:
- `agent.py` — entry point with `run()` function
- `databricks.yml` — DABs bundle configuration
- `CLAUDE.md` — agent-specific instructions for Claude Code
- `requirements.txt` — Python dependencies

Notes:
- Agent names are slugified for the directory/package name. Spaces → underscores.
- `--no-generate` skips the Claude API call and writes placeholder content.
- Known issue: hyphenated slugs produce invalid Python package names (bug-fix branch).
- Known issue: `system_prompt` and `claude_md` are generated in Rust but not yet
  surfaced through the PyO3 binding (bug-fix branch).
