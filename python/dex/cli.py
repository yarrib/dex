"""dex CLI — built on click, extensible by design.

Usage as a standalone CLI:
    dex init --template default

Usage as a framework for org CLIs:
    from dex.cli import create_cli
    cli = create_cli(name="acme-dex", passthroughs=[...])
"""

from __future__ import annotations

from pathlib import Path

import click
from rich.console import Console

from dex.config import DexConfig, load_config, resolve_remote
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

    def get_command(self, ctx: click.Context, cmd_name: str) -> click.Command | None:
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


def _collect_templates(
    extra_dir: Path | None = None,
) -> dict[str, tuple[str, str]]:
    """Return a mapping of template_name -> (source_path, description).

    Sources are checked in priority order; later sources override earlier ones:
      1. Embedded (lowest priority)
      2. User / project config: templates_dir, then remotes
      3. extra_dir from create_cli() (highest priority)

    ``source_path`` is ``"__embedded__"`` for built-ins or an absolute
    directory path string for directory-based templates.
    """
    from dex._core import list_embedded_templates, parse_template_manifest

    registry: dict[str, tuple[str, str]] = {}

    # 1. Embedded templates
    for t in list_embedded_templates():
        registry[t.name] = ("__embedded__", t.description)

    # 2. Config-based sources
    config: DexConfig = load_config()
    dirs_to_scan: list[Path] = []

    if config.templates_dir:
        dirs_to_scan.append(config.templates_dir)

    for remote in config.remotes:
        try:
            with console.status(f"[dim]Updating template remote '{remote.name}'…"):
                local = resolve_remote(remote)
            dirs_to_scan.append(local)
        except RuntimeError as exc:
            console.print(f"[yellow]Warning:[/yellow] {exc}")

    # 3. Programmatic extra_dir (from create_cli)
    if extra_dir:
        dirs_to_scan.append(extra_dir)

    for dir_path in dirs_to_scan:
        if not dir_path.is_dir():
            continue
        for entry in sorted(dir_path.iterdir()):
            manifest_path = entry / "template.toml"
            if entry.is_dir() and manifest_path.exists():
                try:
                    m = parse_template_manifest(str(manifest_path))
                    registry[m.name] = (str(dir_path), m.description)
                except Exception:
                    pass

    return registry


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
@click.pass_context
def init_command(ctx: click.Context, template: str, directory: str, no_prompt: bool) -> None:
    """Scaffold a new project from a template."""
    from dex._core import get_template_variables, scaffold_project

    target = Path(directory).resolve()

    # Collect templates from all sources (embedded + config + create_cli extra_dir).
    extra_dir: Path | None = getattr(ctx.find_root().command, "templates_dir", None)
    registry = _collect_templates(extra_dir=extra_dir)

    console.print(f"\n[bold]dex init[/bold] — scaffolding with template [cyan]{template}[/cyan]\n")

    if template not in registry:
        console.print(f"[red]Error:[/red] template '{template}' not found.")
        console.print(f"Available templates: {', '.join(sorted(registry))}")
        raise SystemExit(1)

    source_path, _ = registry[template]

    # Collect variables from template manifest, then prompt interactively.
    specs = get_template_variables(source_path, template)
    default_project_name = target.name if target.name != "." else Path.cwd().name
    variables: dict[str, object] = {}

    for spec in sorted(specs, key=lambda s: s.name):
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

    result = scaffold_project(source_path, template, str(target), variables)

    console.print(f"\n[green]Scaffolded {len(result.files_created)} files:[/green]")
    for f in sorted(result.files_created):
        console.print(f"  {f}")
    console.print()


@click.group("mcp")
def mcp_group() -> None:
    """MCP server for AI agent integration."""


@mcp_group.command("serve")
def mcp_serve() -> None:  # pragma: no cover
    """Start the dex MCP server (stdio transport)."""
    import asyncio

    from dex.mcp_server import serve

    asyncio.run(serve())


# Default CLI instance for `dex` command.
cli = create_cli()
