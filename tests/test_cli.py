"""Smoke tests for the dex CLI."""

from pathlib import Path

import pytest
from click.testing import CliRunner

from dex.cli import cli


def test_cli_help() -> None:
    result = CliRunner().invoke(cli, ["--help"])
    assert result.exit_code == 0
    assert "dex" in result.output


def test_cli_version() -> None:
    result = CliRunner().invoke(cli, ["--version"])
    assert result.exit_code == 0


def test_init_help() -> None:
    result = CliRunner().invoke(cli, ["init", "--help"])
    assert result.exit_code == 0


def test_agent_help() -> None:
    result = CliRunner().invoke(cli, ["agent", "--help"])
    assert result.exit_code == 0


def test_mcp_help() -> None:
    result = CliRunner().invoke(cli, ["mcp", "--help"])
    assert result.exit_code == 0


def test_init_unknown_template_exits_nonzero(tmp_path: Path) -> None:
    result = CliRunner().invoke(
        cli, ["init", "--template", "does-not-exist", "--dir", str(tmp_path), "--no-prompt"]
    )
    assert result.exit_code != 0


def test_init_default_template_no_prompt(tmp_path: Path) -> None:
    result = CliRunner().invoke(
        cli, ["init", "--template", "default", "--dir", str(tmp_path), "--no-prompt"]
    )
    assert result.exit_code == 0, result.output
    assert "Scaffolded" in result.output
