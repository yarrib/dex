"""Tests for passthrough command support and the public extension API."""

from __future__ import annotations

from unittest.mock import MagicMock, patch

from click.testing import CliRunner

from dex.cli import create_cli, passthrough
from dex.passthrough import PassthroughCommand, PassthroughSpec


# ---------------------------------------------------------------------------
# dex.ext public API
# ---------------------------------------------------------------------------


def test_ext_exports() -> None:
    import dex.ext as ext

    assert hasattr(ext, "create_cli")
    assert hasattr(ext, "passthrough")
    assert hasattr(ext, "PassthroughSpec")
    assert hasattr(ext, "DexGroup")


# ---------------------------------------------------------------------------
# PassthroughSpec
# ---------------------------------------------------------------------------


class TestPassthroughSpec:
    def test_fields(self) -> None:
        spec = PassthroughSpec(name="db", command="databricks", description="Databricks CLI")
        assert spec.name == "db"
        assert spec.command == "databricks"
        assert spec.description == "Databricks CLI"

    def test_description_optional(self) -> None:
        spec = PassthroughSpec(name="db", command="databricks")
        assert spec.description is None


# ---------------------------------------------------------------------------
# PassthroughCommand
# ---------------------------------------------------------------------------


class TestPassthroughCommand:
    def test_creation_sets_target(self) -> None:
        cmd = PassthroughCommand(name="test", target_command="echo")
        assert cmd.target_command == "echo"

    def test_get_short_help_str_with_description(self) -> None:
        cmd = PassthroughCommand(name="test", target_command="echo", description="echo stuff")
        assert cmd.get_short_help_str() == "echo stuff"

    def test_get_short_help_str_default(self) -> None:
        cmd = PassthroughCommand(name="test", target_command="echo")
        assert "echo" in cmd.get_short_help_str()

    def test_invoke_delegates_to_subprocess(self) -> None:
        cmd = PassthroughCommand(name="test", target_command="echo")
        with patch("dex.passthrough.subprocess.run") as mock_run:
            mock_run.return_value = MagicMock(returncode=0)
            result = CliRunner().invoke(cmd, ["hello", "world"])
        assert result.exit_code == 0
        mock_run.assert_called_once()
        called_args = mock_run.call_args[0][0]
        assert called_args[0] == "echo"
        assert "hello" in called_args

    def test_invoke_propagates_returncode(self) -> None:
        cmd = PassthroughCommand(name="test", target_command="false")
        with patch("dex.passthrough.subprocess.run") as mock_run:
            mock_run.return_value = MagicMock(returncode=1)
            result = CliRunner().invoke(cmd, [])
        assert result.exit_code == 1


# ---------------------------------------------------------------------------
# passthrough() factory + DexGroup resolution
# ---------------------------------------------------------------------------


class TestPassthroughFactory:
    def test_returns_passthrough_spec(self) -> None:
        spec = passthrough("db", "databricks", "Databricks CLI")
        assert isinstance(spec, PassthroughSpec)
        assert spec.name == "db"
        assert spec.command == "databricks"

    def test_description_forwarded(self) -> None:
        spec = passthrough("db", "databricks", "desc")
        assert spec.description == "desc"


class TestDexGroupPassthrough:
    def _cli_with_echo(self):
        return create_cli(
            name="test-cli",
            passthroughs=[passthrough("echo-cmd", "echo", "Echo passthrough")],
        )

    def test_passthrough_appears_in_help(self) -> None:
        result = CliRunner().invoke(self._cli_with_echo(), ["--help"])
        assert result.exit_code == 0
        assert "echo-cmd" in result.output

    def test_passthrough_resolved_and_invoked(self) -> None:
        cli = self._cli_with_echo()
        with patch("dex.passthrough.subprocess.run") as mock_run:
            mock_run.return_value = MagicMock(returncode=0)
            result = CliRunner().invoke(cli, ["echo-cmd", "hello"])
        assert result.exit_code == 0
        mock_run.assert_called_once()

    def test_unknown_command_returns_none(self) -> None:
        from dex.cli import DexGroup
        import click

        @click.group(cls=DexGroup, invoke_without_command=True)
        def g():
            pass

        with g.make_context("g", []) as ctx:
            assert g.get_command(ctx, "nonexistent") is None
