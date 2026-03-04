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


def passthrough(name: str, command: str, description: str | None = None) -> PassthroughSpec:
    """Create a PassthroughSpec. Sugar for use with create_cli().

    Example::

        cli = create_cli(
            name="acme-dex",
            passthroughs=[
                passthrough("db", "databricks", "Databricks CLI"),
            ],
        )
    """
    return PassthroughSpec(name=name, command=command, description=description)


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
        """Extensible CLI for data project operations."""

    # Store config on the group for subcommands to access.
    group.templates_dir = templates_dir  # type: ignore[attr-defined]
    group.config_defaults = config_defaults or {}  # type: ignore[attr-defined]

    # Register built-in commands.
    group.add_command(init_command)

    # Register agent subcommand group.
    from dex.agent import agent_group

    group.add_command(agent_group)

    # Register MCP subcommand group.
    group.add_command(mcp_group)

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
    from dex._core import get_template_variables, list_embedded_templates, scaffold_project

    target = Path(directory).resolve()

    console.print(f"\n[bold]dex init[/bold] — scaffolding with template [cyan]{template}[/cyan]\n")

    templates = list_embedded_templates()
    template_names = [t.name for t in templates]

    if template not in template_names:
        console.print(f"[red]Error:[/red] template '{template}' not found.")
        console.print(f"Available templates: {', '.join(template_names)}")
        raise SystemExit(1)

    # Collect variables from template manifest, then prompt interactively.
    specs = get_template_variables("__embedded__", template)
    default_project_name = target.name if target.name != "." else Path.cwd().name
    variables: dict[str, object] = {}

    for spec in sorted(specs, key=lambda s: s.name):
        # For project_name, use the target directory name as the default.
        effective_default = (
            default_project_name if spec.name == "project_name" else spec.default or ""
        )

        if no_prompt:
            if spec.var_type == "bool":
                variables[spec.name] = (spec.default or "true") == "true"
            elif spec.var_type == "choice":
                variables[spec.name] = effective_default or (spec.choices or [""])[0]
            else:
                variables[spec.name] = effective_default
        elif spec.var_type == "choice":
            choices = spec.choices or []
            variables[spec.name] = click.prompt(
                spec.prompt,
                type=click.Choice(choices),
                default=effective_default or choices[0],
            )
        elif spec.var_type == "bool":
            variables[spec.name] = click.confirm(
                spec.prompt,
                default=(spec.default or "true") == "true",
            )
        else:
            variables[spec.name] = click.prompt(spec.prompt, default=effective_default)

    result = scaffold_project("__embedded__", template, str(target), variables)

    console.print(f"\n[green]Scaffolded {len(result.files_created)} files:[/green]")
    for f in sorted(result.files_created):
        console.print(f"  {f}")
    console.print()


def _run_dabs_init(
    dabs_source: str,
    target_dir: Path,
    variables: dict[str, str],
    variable_map: dict[str, str],
) -> None:
    """Phase 1: delegate to `databricks bundle init` for DABs-composite templates.

    Writes a temporary config JSON mapping dex variables to DABs variable names,
    then runs `databricks bundle init` non-interactively.
    """
    import json
    import shutil
    import subprocess
    import tempfile

    # Check that databricks CLI is available.
    if shutil.which("databricks") is None:
        console.print(
            "[red]Error:[/red] this template requires the Databricks CLI "
            "(`databricks`), but it was not found on PATH."
        )
        console.print("Install it: https://docs.databricks.com/dev-tools/cli/install.html")
        raise SystemExit(1)

    # Map dex variable names → DABs variable names.
    dabs_config = {}
    for dex_var, dabs_var in variable_map.items():
        if dex_var in variables:
            dabs_config[dabs_var] = variables[dex_var]

    # Write config to a temp file and run databricks bundle init.
    with tempfile.NamedTemporaryFile(
        mode="w", suffix=".json", delete=False, prefix="dex-dabs-"
    ) as f:
        json.dump(dabs_config, f)
        config_path = f.name

    try:
        console.print("[dim]Running databricks bundle init...[/dim]")
        result = subprocess.run(
            [
                "databricks",
                "bundle",
                "init",
                dabs_source,
                "--output-dir",
                str(target_dir),
                "--config-file",
                config_path,
            ],
            capture_output=True,
            text=True,
        )
        if result.returncode != 0:
            console.print(f"[red]Error:[/red] databricks bundle init failed:\n{result.stderr}")
            raise SystemExit(result.returncode)
        console.print("[green]DABs template scaffolded.[/green]")
    finally:
        Path(config_path).unlink(missing_ok=True)


@click.group("mcp")
def mcp_group() -> None:
    """MCP server for AI agent integration."""


@mcp_group.command("serve")
def mcp_serve() -> None:
    """Start the dex MCP server (stdio transport)."""
    import asyncio

    from dex.mcp_server import serve

    asyncio.run(serve())


# Default CLI instance for `dex` command.
cli = create_cli()
