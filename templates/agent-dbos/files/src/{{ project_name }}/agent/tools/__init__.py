"""Tool registry. Add a tool by defining a `Tool` and listing it in `TOOLS`."""

from __future__ import annotations

from collections.abc import Callable
from dataclasses import dataclass
from typing import Any

from .example_tool import record_note


@dataclass(frozen=True)
class Tool:
    name: str
    description: str
    parameters: dict[str, Any]  # JSON Schema for the arguments
    # fn(arguments, idempotency_key) -> JSON-serializable dict. Runs inside a DBOS step,
    # so it may be re-executed after a crash: make the side effect idempotent by key.
    fn: Callable[[dict[str, Any], str], dict[str, Any]]
    needs_approval: bool = False


TOOLS: dict[str, Tool] = {
    "record_note": Tool(
        name="record_note",
        description="Record a note in the ledger (an example side effect).",
        parameters={
            "type": "object",
            "properties": {"text": {"type": "string", "description": "The note text."}},
            "required": ["text"],
        },
        fn=record_note,
        needs_approval=True,
    ),
}
