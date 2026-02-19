"""dex CLI — built on click, extensible by design.

Usage as a standalone CLI:
    dex init --template default

Usage as a framework for org CLIs:
    from dex.cli import create_cli
    cli = create_cli(name="acme-dex", passthroughs=[...])
"""

from __future__ import annotations

import os
from pathlib import Path

import click
from rich.console import Console

from dex.passthrough import PassthroughCommand, PassthroughSpec

console = Console()


class DexGroup(click.Group):
    """click Group subclass that supports pass-through commands and plugin discovery."""

    def __init__(self, passthroughs: dict[str, PassthroughSpec] | None = None, **kwargs):
        super().__init__(**kwargs)
        self._passthroughs = passthroughs or {}

    def get_command(self, ctx: click.Context, cmd_name: str) -> click.BaseCommand | None:
        # 1. Built-in commands
        rv = super().get_command(ctx, cmd_name)
        if rv is not None:
            return rv

        # 2. Pass-through commands
        if cmd_name in self._passthroughs:
            spec = self._passthroughs[cmd_name]
            return PassthroughCommand(
                name=cmd_name,
                target_command=spec.command,
                description=spec.description,
            )

        return None

    def list_commands(self, ctx: click.Context) -> list[str]:
        builtins = super().list_commands(ctx)
        passthroughs = sorted(self._passthroughs.keys())
        return builtins + passthroughs


def create_cli(
    name: str = "dex",
    templates_dir: str | Path | None = None,
    config_defaults: dict[str, str] | None = None,
    passthroughs: list[PassthroughSpec] | None = None,
) -> DexGroup:
    """Factory for creating a dex CLI instance.

    Teams call this to build their org-specific CLI::

        cli = create_cli(name="acme-dex", passthroughs=[
            PassthroughSpec(name="db", command="databricks", description="Databricks CLI"),
        ])

        @cli.command()
        def my_custom_command():
            ...
    """
    pt_map = {p.name: p for p in (passthroughs or [])}

    @click.group(name=name, cls=DexGroup, passthroughs=pt_map)
    @click.version_option(package_name="dex")
    def group():
        """Opinionated CLI for Databricks/MLOps project operations."""

    # Store config on the group for subcommands to access.
    group.templates_dir = templates_dir  # type: ignore[attr-defined]
    group.config_defaults = config_defaults or {}  # type: ignore[attr-defined]

    # Register built-in commands.
    group.add_command(init_command)

    return group


@click.command("init")
@click.option(
    "--template",
    "-t",
    default="default",
    show_default=True,
    help="Template to scaffold from.",
)
@click.option(
    "--dir",
    "-d",
    "directory",
    default=".",
    type=click.Path(),
    help="Target directory.",
)
@click.option(
    "--no-prompt",
    is_flag=True,
    help="Use defaults for all variables (non-interactive).",
)
def init_command(template: str, directory: str, no_prompt: bool) -> None:
    """Scaffold a new project from a template."""
    from dex._core import list_embedded_templates, scaffold_project

    target = Path(directory).resolve()

    console.print(f"\n[bold]dex init[/bold] — scaffolding with template [cyan]{template}[/cyan]\n")

    # For v0.1, scaffold from embedded templates.
    # Future: resolve template from multiple sources.
    templates = list_embedded_templates()
    template_names = [t.name for t in templates]

    if template not in template_names:
        console.print(f"[red]Error:[/red] template '{template}' not found.")
        console.print(f"Available templates: {', '.join(template_names)}")
        raise SystemExit(1)

    # Collect variables interactively.
    # For v0.1 with embedded templates, we parse the manifest via the core.
    variables = {}

    if not no_prompt:
        # Default variable: project_name from target directory name
        default_name = target.name if target.name != "." else Path.cwd().name
        project_name = click.prompt("Project name", default=default_name)
        variables["project_name"] = project_name
    else:
        variables["project_name"] = target.name if target.name != "." else Path.cwd().name

    # Scaffold
    result = scaffold_project("__embedded__", template, str(target), variables)

    console.print(f"\n[green]Scaffolded {len(result.files_created)} files:[/green]")
    for f in sorted(result.files_created):
        console.print(f"  {f}")
    console.print()


# Default CLI instance for `dex` command.
cli = create_cli()
