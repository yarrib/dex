"""dex MCP server — AI agent integration via Model Context Protocol.

Exposes dex operations as MCP tools so AI agents (Claude Code, Codex, etc.)
can scaffold projects and query templates without a human in the loop.

Transport: stdio (run with `dex mcp serve`).

Tools exposed:
    list_templates          — list available templates (fully implemented)
    scaffold_project        — scaffold a project from a template (stub)
    scaffold_agent          — scaffold an agent from a template (stub)
    get_template_variables  — list variables for a template (stub)
"""

from __future__ import annotations

import json
from typing import Any

from mcp.server import Server
from mcp.server.stdio import stdio_server
from mcp.types import TextContent, Tool

# ---------------------------------------------------------------------------
# Tool definitions
# ---------------------------------------------------------------------------

_TOOLS: list[Tool] = [
    Tool(
        name="list_templates",
        description=(
            "List all templates available in dex. "
            "Returns each template's name, description, and version."
        ),
        inputSchema={
            "type": "object",
            "properties": {},
            "required": [],
        },
    ),
    Tool(
        name="scaffold_project",
        description=(
            "Scaffold a new project from a dex template into the given directory. "
            "All required template variables must be supplied."
        ),
        inputSchema={
            "type": "object",
            "properties": {
                "template": {
                    "type": "string",
                    "description": (
                        "Template name (e.g. 'default'). Use list_templates to discover names."
                    ),
                },
                "directory": {
                    "type": "string",
                    "description": (
                        "Absolute path to the target directory. Created if it does not exist."
                    ),
                },
                "variables": {
                    "type": "object",
                    "description": "Key-value map of template variable values.",
                    "additionalProperties": {"type": "string"},
                },
            },
            "required": ["template", "directory"],
        },
    ),
    Tool(
        name="scaffold_agent",
        description=(
            "Scaffold a new AI agent from a dex agent template. "
            "Returns the paths of generated files."
        ),
        inputSchema={
            "type": "object",
            "properties": {
                "name": {
                    "type": "string",
                    "description": "Human-readable agent name (e.g. 'Table Anomaly Monitor').",
                },
                "description": {
                    "type": "string",
                    "description": (
                        "What the agent does. Used to generate system prompt and CLAUDE.md."
                    ),
                },
                "directory": {
                    "type": "string",
                    "description": "Absolute path to the target directory.",
                },
                "generate": {
                    "type": "boolean",
                    "description": (
                        "If true, generate system prompt and CLAUDE.md via LLM. Default false."
                    ),
                    "default": False,
                },
            },
            "required": ["name", "directory"],
        },
    ),
    Tool(
        name="get_template_variables",
        description=(
            "Return the variable specification for a template: "
            "name, prompt text, type, default value, and whether it is required."
        ),
        inputSchema={
            "type": "object",
            "properties": {
                "template": {
                    "type": "string",
                    "description": "Template name. Use list_templates to discover names.",
                },
            },
            "required": ["template"],
        },
    ),
]

# ---------------------------------------------------------------------------
# Tool handlers
# ---------------------------------------------------------------------------


def _handle_list_templates() -> list[dict[str, Any]]:
    from dex._core import list_embedded_templates

    templates = list_embedded_templates()
    return [
        {
            "name": t.name,
            "description": t.description,
            "version": t.version,
        }
        for t in templates
    ]


def _handle_scaffold_project(
    template: str, directory: str, variables: dict[str, str] | None
) -> dict[str, Any]:
    raise NotImplementedError(
        "scaffold_project is not yet implemented in the MCP server. "
        "Use `dex init` from the CLI instead."
    )


def _handle_scaffold_agent(
    name: str, directory: str, description: str | None, generate: bool
) -> dict[str, Any]:
    raise NotImplementedError(
        "scaffold_agent is not yet implemented in the MCP server. "
        "Use `dex agent new` from the CLI instead."
    )


def _handle_get_template_variables(template: str) -> list[dict[str, Any]]:
    raise NotImplementedError(
        "get_template_variables is not yet implemented in the MCP server. "
        "See template manifests in the templates/ directory."
    )


# ---------------------------------------------------------------------------
# Server entry point
# ---------------------------------------------------------------------------


async def serve() -> None:  # pragma: no cover
    """Run the dex MCP server over stdio."""
    server = Server("dex")

    @server.list_tools()
    async def list_tools() -> list[Tool]:
        return _TOOLS

    @server.call_tool()
    async def call_tool(name: str, arguments: dict[str, Any]) -> list[TextContent]:
        try:
            if name == "list_templates":
                result = _handle_list_templates()
            elif name == "scaffold_project":
                result = _handle_scaffold_project(
                    template=arguments["template"],
                    directory=arguments["directory"],
                    variables=arguments.get("variables"),
                )
            elif name == "scaffold_agent":
                result = _handle_scaffold_agent(
                    name=arguments["name"],
                    directory=arguments["directory"],
                    description=arguments.get("description"),
                    generate=bool(arguments.get("generate", False)),
                )
            elif name == "get_template_variables":
                result = _handle_get_template_variables(template=arguments["template"])
            else:
                raise ValueError(f"Unknown tool: {name!r}")
        except NotImplementedError as exc:
            return [TextContent(type="text", text=f"[not implemented] {exc}")]
        except Exception as exc:
            return [TextContent(type="text", text=f"[error] {exc}")]

        return [TextContent(type="text", text=json.dumps(result, indent=2))]

    async with stdio_server() as (read_stream, write_stream):
        await server.run(read_stream, write_stream, server.create_initialization_options())
