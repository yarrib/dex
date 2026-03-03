"""dex agent — opinionated agent project scaffolding for Databricks.

Combines deterministic scaffolding (Rust core) with an optional generative
phase (Claude API) to produce working, deployable agent projects.
"""

from __future__ import annotations

from pathlib import Path

import click
from rich.console import Console
from rich.panel import Panel
from rich.prompt import Confirm, Prompt

console = Console()

# Maps user-friendly trigger names to the Rust enum values.
TRIGGER_CHOICES = {
    "user request": "user_request",
    "schedule": "schedule",
    "event": "event",
    "upstream system": "upstream_system",
}

DEPLOY_CHOICES = {
    "job": "job",
    "serving endpoint": "serving_endpoint",
    "interactive": "interactive",
}


@click.group("agent")
def agent_group():
    """Agent project scaffolding for Databricks."""


@agent_group.command("new")
@click.option("--name", "-n", default=None, help="Agent name (skips first prompt).")
@click.option("--dir", "-d", "directory", default=".", help="Parent directory.")
@click.option("--no-generate", is_flag=True, help="Skip the generative phase (Claude).")
def agent_new(name: str | None, directory: str, no_generate: bool) -> None:
    """Scaffold a new agent project via interactive Q&A."""
    from dex._core import scaffold_agent

    console.print(Panel("[bold]dex agent new[/bold]", style="cyan"))
    console.print()

    # --- Q&A Flow ---
    if name is None:
        name = Prompt.ask("[bold]What does this agent do in one sentence?[/bold]")

    agent_name = Prompt.ask("Agent name", default=_suggest_name(name)) if name else name

    description = name  # The one-sentence description IS the name prompt answer.
    if agent_name != name:
        description = name  # They gave a description first, then a name.

    trigger = _prompt_choice(
        "What triggers it?",
        list(TRIGGER_CHOICES.keys()),
    )

    success = Prompt.ask("[bold]What does success look like?[/bold]")

    reads = Prompt.ask("[bold]What does it need to read?[/bold]")
    writes = Prompt.ask("[bold]What does it need to write or change?[/bold]")

    handoff = Confirm.ask("Does it hand off to a human or another agent?", default=False)
    autonomous = Confirm.ask("Should it act autonomously?", default=True)

    example_input = Prompt.ask("[bold]Example input[/bold]")
    example_output = Prompt.ask("[bold]What should the correct behavior/output be?[/bold]")
    bad_output = Prompt.ask("[bold]What would a bad or dangerous output look like?[/bold]")

    deploy_target = _prompt_choice(
        "How should it be deployed?",
        list(DEPLOY_CHOICES.keys()),
    )

    # --- Scaffold (deterministic phase) ---
    console.print()
    target = Path(directory).resolve()

    answers = {
        "name": agent_name,
        "description": description,
        "trigger": TRIGGER_CHOICES[trigger],
        "success_criteria": success,
        "reads": reads,
        "writes": writes,
        "handoff": handoff,
        "autonomous": autonomous,
        "example_input": example_input,
        "example_output": example_output,
        "bad_output": bad_output,
        "deploy_target": DEPLOY_CHOICES[deploy_target],
    }

    with console.status("[bold cyan]Scaffolding agent project...[/bold cyan]"):
        result = scaffold_agent(answers, str(target))

    console.print(f"\n[green]Scaffolded {len(result.files_created)} files:[/green]")
    for f in sorted(result.files_created):
        console.print(f"  {f}")

    # --- Generative phase (future) ---
    if not no_generate:
        console.print(
            "\n[dim]Generative phase (Claude API) not yet implemented. "
            "Use --no-generate or flesh out agent.py manually.[/dim]"
        )

    project_dir = Path(result.project_dir)
    console.print(f"\n[bold green]Done.[/bold green] cd {project_dir.name} && dex deploy\n")


def _suggest_name(description: str) -> str:
    """Suggest a project name from a description."""
    words = description.lower().split()
    # Take first 3-4 meaningful words, skip common filler.
    skip = {"a", "an", "the", "and", "or", "for", "to", "in", "on", "that", "which", "is"}
    meaningful = [w for w in words if w not in skip][:4]
    return "-".join(meaningful).rstrip(".")


def _prompt_choice(question: str, choices: list[str]) -> str:
    """Prompt user to select from a list of choices."""
    console.print(f"[bold]{question}[/bold]")
    for i, choice in enumerate(choices, 1):
        console.print(f"  {i}. {choice}")
    while True:
        answer = Prompt.ask("Select", default="1")
        try:
            idx = int(answer) - 1
            if 0 <= idx < len(choices):
                return choices[idx]
        except ValueError:
            # Try matching by name.
            for choice in choices:
                if answer.lower() in choice.lower():
                    return choice
        console.print(f"[red]Please enter 1-{len(choices)}[/red]")
