---
id: mcp
title: mcp
kind: module
summary: Model Context Protocol support — backs `dex mcp serve` for exposing dex to AI agents.
related:
  - dex-core: part-of
  - error: uses
  - cli-commands: exposed-by
---

`crates/dex-core/src/mcp.rs` provides the Model Context Protocol functionality
that backs the `dex mcp serve` command. It lets AI agents/tools interact with
dex capabilities over MCP.

As with the rest of `dex-core`, it returns data and propagates errors; the CLI
(`commands/mcp.rs`) handles process/serving concerns and user-facing output.
