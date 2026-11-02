"""Pydantic models shared by workflow, steps and API.

Steps return `.model_dump()` of these — plain JSON-serializable dicts — so DBOS can
checkpoint them. Never return a model instance or an arbitrary object from a step.
"""

from __future__ import annotations

from typing import Any, Literal

from pydantic import BaseModel, Field


class ToolCall(BaseModel):
    id: str
    name: str
    arguments: dict[str, Any] = Field(default_factory=dict)
    needs_approval: bool = False


class Completion(BaseModel):
    """One model turn: text and/or tool calls."""

    text: str = ""
    tool_calls: list[ToolCall] = Field(default_factory=list)

    @property
    def done(self) -> bool:
        return not self.tool_calls

    @property
    def summary(self) -> str:
        if self.tool_calls:
            return "calls " + ", ".join(c.name for c in self.tool_calls)
        return (self.text[:120] + "…") if len(self.text) > 120 else self.text


class Decision(BaseModel):
    approved: bool
    reason: str = ""
    source: Literal["human", "timeout"] = "human"


class RunResult(BaseModel):
    status: Literal["completed", "rejected", "turn_budget_exceeded"] = "completed"
    output: str = ""
    turns: int = 0
