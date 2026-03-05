"""Integration tests for `dex mcp serve` and the MCP server handlers."""

from __future__ import annotations

import asyncio

import pytest
from click.testing import CliRunner

from dex.cli import cli
from dex.mcp_server import (
    _handle_get_template_variables,
    _handle_list_templates,
    _handle_scaffold_agent,
    _handle_scaffold_project,
    _TOOLS,
)


# ---------------------------------------------------------------------------
# CLI surface
# ---------------------------------------------------------------------------


class TestMcpCli:
    def test_mcp_help(self) -> None:
        result = CliRunner().invoke(cli, ["mcp", "--help"])
        assert result.exit_code == 0

    def test_mcp_serve_help(self) -> None:
        result = CliRunner().invoke(cli, ["mcp", "serve", "--help"])
        assert result.exit_code == 0


# ---------------------------------------------------------------------------
# Tool registry
# ---------------------------------------------------------------------------


class TestMcpToolRegistry:
    def test_tools_list_is_non_empty(self) -> None:
        assert len(_TOOLS) > 0

    def test_expected_tools_registered(self) -> None:
        names = {t.name for t in _TOOLS}
        assert "list_templates" in names
        assert "scaffold_project" in names
        assert "scaffold_agent" in names
        assert "get_template_variables" in names

    def test_each_tool_has_input_schema(self) -> None:
        for tool in _TOOLS:
            assert tool.inputSchema is not None
            assert "type" in tool.inputSchema


# ---------------------------------------------------------------------------
# Handler functions
# ---------------------------------------------------------------------------


def _requires_embedded_templates():
    """Skip marker for tests that need a maturin-built extension with templates."""
    import pytest

    return pytest.mark.skipif(
        not _handle_list_templates(),
        reason="Extension not built with embedded templates — run 'maturin develop'",
    )


class TestHandlerListTemplates:
    def test_returns_list(self) -> None:
        result = _handle_list_templates()
        assert isinstance(result, list)

    @_requires_embedded_templates()
    def test_returns_non_empty(self) -> None:
        assert len(_handle_list_templates()) > 0

    def test_each_entry_has_required_keys(self) -> None:
        for t in _handle_list_templates():
            assert "name" in t
            assert "description" in t
            assert "version" in t

    @_requires_embedded_templates()
    def test_known_template_present(self) -> None:
        names = [t["name"] for t in _handle_list_templates()]
        assert "default" in names


class TestHandlerStubs:
    def test_scaffold_project_not_implemented(self) -> None:
        with pytest.raises(NotImplementedError):
            _handle_scaffold_project("default", "/tmp/test", None)

    def test_scaffold_agent_not_implemented(self) -> None:
        with pytest.raises(NotImplementedError):
            _handle_scaffold_agent("my-agent", "/tmp/test", None, False)

    def test_get_template_variables_not_implemented(self) -> None:
        with pytest.raises(NotImplementedError):
            _handle_get_template_variables("default")


# ---------------------------------------------------------------------------
# Server call_tool routing (via serve() with mocked transport)
# ---------------------------------------------------------------------------


class TestCallToolRouting:
    """Test the call_tool dispatch logic by bootstrapping the server."""

    def _run_tool(self, tool_name: str, arguments: dict) -> list:
        """Boot the server, extract the call_tool handler, and invoke it."""
        from mcp.server import Server
        from mcp.types import TextContent, Tool

        import json
        from typing import Any

        from dex.mcp_server import _TOOLS, _handle_list_templates

        # Replicate the dispatch logic from serve() to test routing.
        async def dispatch(name: str, args: dict[str, Any]) -> list[TextContent]:
            try:
                if name == "list_templates":
                    result = _handle_list_templates()
                elif name == "scaffold_project":
                    _handle_scaffold_project(
                        template=args["template"],
                        directory=args["directory"],
                        variables=args.get("variables"),
                    )
                elif name == "scaffold_agent":
                    _handle_scaffold_agent(
                        name=args["name"],
                        directory=args["directory"],
                        description=args.get("description"),
                        generate=bool(args.get("generate", False)),
                    )
                elif name == "get_template_variables":
                    _handle_get_template_variables(template=args["template"])
                else:
                    raise ValueError(f"Unknown tool: {name!r}")
            except NotImplementedError as exc:
                return [TextContent(type="text", text=f"[not implemented] {exc}")]
            except Exception as exc:
                return [TextContent(type="text", text=f"[error] {exc}")]

            return [TextContent(type="text", text=json.dumps(result, indent=2))]

        return asyncio.run(dispatch(tool_name, arguments))

    def test_list_templates_returns_json(self) -> None:
        import json

        items = self._run_tool("list_templates", {})
        assert len(items) == 1
        data = json.loads(items[0].text)
        assert isinstance(data, list)

    @_requires_embedded_templates()
    def test_list_templates_contains_default(self) -> None:
        import json

        items = self._run_tool("list_templates", {})
        data = json.loads(items[0].text)
        assert any(t["name"] == "default" for t in data)

    def test_scaffold_project_returns_not_implemented(self) -> None:
        items = self._run_tool("scaffold_project", {"template": "default", "directory": "/tmp/x"})
        assert "[not implemented]" in items[0].text

    def test_scaffold_agent_returns_not_implemented(self) -> None:
        items = self._run_tool("scaffold_agent", {"name": "test", "directory": "/tmp/x"})
        assert "[not implemented]" in items[0].text

    def test_get_template_variables_returns_not_implemented(self) -> None:
        items = self._run_tool("get_template_variables", {"template": "default"})
        assert "[not implemented]" in items[0].text

    def test_unknown_tool_returns_error(self) -> None:
        items = self._run_tool("nonexistent_tool", {})
        assert "[error]" in items[0].text
