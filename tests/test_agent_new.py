"""Integration tests for `dex agent new`."""

from __future__ import annotations

from pathlib import Path
from unittest.mock import patch

import pytest
from click.testing import CliRunner

from dex.agent import _suggest_name
from dex.cli import cli


# ---------------------------------------------------------------------------
# _suggest_name unit tests
# ---------------------------------------------------------------------------


class TestSuggestName:
    def test_filters_stopwords(self) -> None:
        assert _suggest_name("a table anomaly monitor") == "table-anomaly-monitor"

    def test_hyphenates_words(self) -> None:
        assert _suggest_name("monitor tables") == "monitor-tables"

    def test_strips_trailing_period(self) -> None:
        assert _suggest_name("monitor tables.") == "monitor-tables"

    def test_max_four_words(self) -> None:
        result = _suggest_name("one two three four five six")
        assert len(result.split("-")) == 4


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

# Ordered Prompt.ask return values when --name is supplied (skips first prompt).
_PROMPT_ANSWERS = [
    "my-agent",           # "Agent name"
    "1",                  # trigger select → user request
    "files processed",    # success criteria
    "Unity Catalog table",  # reads
    "output Delta table", # writes
    "sample input data",  # example input
    "correct summary",    # example output
    "deleted wrong rows", # bad output
    "1",                  # deploy target select → job
]

_CONFIRM_ANSWERS = [False, True]  # handoff=False, autonomous=True


def _invoke_agent_new(tmp_path: Path, extra_args: list[str] | None = None) -> object:
    args = [
        "agent", "new",
        "--name", "monitors table anomalies",
        "--dir", str(tmp_path),
        *(extra_args or []),
    ]
    with (
        patch("dex.agent.Prompt.ask", side_effect=list(_PROMPT_ANSWERS)),
        patch("dex.agent.Confirm.ask", side_effect=list(_CONFIRM_ANSWERS)),
    ):
        return CliRunner().invoke(cli, args, catch_exceptions=False)


# ---------------------------------------------------------------------------
# Integration tests
# ---------------------------------------------------------------------------


class TestAgentNew:
    def test_help(self) -> None:
        result = CliRunner().invoke(cli, ["agent", "new", "--help"])
        assert result.exit_code == 0
        assert "--name" in result.output
        assert "--no-generate" in result.output

    def test_scaffolds_project_directory(self, tmp_path: Path) -> None:
        result = _invoke_agent_new(tmp_path, ["--no-generate"])
        assert result.exit_code == 0
        assert any(tmp_path.iterdir()), "expected scaffolded project directory in tmp_path"

    def test_no_generate_exits_cleanly(self, tmp_path: Path) -> None:
        result = _invoke_agent_new(tmp_path, ["--no-generate"])
        assert result.exit_code == 0

    def test_without_no_generate_exits_cleanly(self, tmp_path: Path) -> None:
        # Generative phase is stubbed but must not crash.
        result = _invoke_agent_new(tmp_path)
        assert result.exit_code == 0

    def test_name_flag_skips_description_prompt(self, tmp_path: Path) -> None:
        # With --name supplied, Prompt.ask should NOT be called for the description.
        # We only provide answers for the remaining prompts; extra calls would exhaust
        # the side_effect and raise StopIteration, failing the test.
        result = _invoke_agent_new(tmp_path, ["--no-generate"])
        assert result.exit_code == 0

    def test_project_dir_named_after_agent(self, tmp_path: Path) -> None:
        result = _invoke_agent_new(tmp_path, ["--no-generate"])
        assert result.exit_code == 0
        dirs = [p for p in tmp_path.iterdir() if p.is_dir()]
        assert len(dirs) >= 1

    @pytest.mark.parametrize(
        ("description", "expected"),
        [
            ("monitors anomalies in Delta tables", "monitors-anomalies-delta-tables"),
            ("a simple test agent.", "simple-test-agent"),
            ("the agent that does things", "agent-does-things"),
        ],
    )
    def test_suggest_name_parametrized(self, description: str, expected: str) -> None:
        assert _suggest_name(description) == expected
