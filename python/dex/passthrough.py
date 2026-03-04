"""Pass-through command support for delegating to external CLIs."""

from __future__ import annotations

import subprocess
import sys
from dataclasses import dataclass

import click


@dataclass
class PassthroughSpec:
    """Specification for a pass-through command."""

    name: str
    command: str
    description: str | None = None


class PassthroughCommand(click.BaseCommand):  # type: ignore[misc]
    """A click command that delegates to an external CLI.

    All arguments after the command name are forwarded to the target CLI.
    stdin/stdout/stderr are inherited for full interactivity.

    Example::

        # dex db clusters list --output json
        # → databricks clusters list --output json
    """

    def __init__(
        self,
        name: str,
        target_command: str,
        description: str | None = None,
        **kwargs,
    ):
        super().__init__(name, **kwargs)
        self.target_command = target_command
        self.help = description or f"Pass-through to `{target_command}`"
        self.context_settings = {"ignore_unknown_options": True, "allow_extra_args": True}

    def parse_args(self, ctx: click.Context, args: list[str]) -> list[str]:
        ctx.args = args
        return args

    def invoke(self, ctx: click.Context) -> None:
        result = subprocess.run(
            [self.target_command, *ctx.args],
            stdin=sys.stdin,
            stdout=sys.stdout,
            stderr=sys.stderr,
        )
        ctx.exit(result.returncode)

    def get_short_help_str(self, limit: int = 150) -> str:
        return self.help or ""
