"""Progress streaming: durable events plus live (non-durable) model tokens.

* Durable progress events (turn summaries, tool results, status) are appended to a DBOS
  workflow stream. Each event has an increasing offset, so SSE clients can reconnect with
  `Last-Event-ID` and lose nothing. Writing an event is checkpointed: a recovered
  workflow does not duplicate events it already wrote.
* Model tokens go over an in-memory bus only. They are visible to clients connected to the
  process executing the run; only the final completion is checkpointed.
"""

from __future__ import annotations

import queue
import threading
from collections.abc import AsyncIterator, Iterator
from typing import Any

from dbos import DBOS

EVENTS_KEY = "events"


def emit(kind: str, data: dict[str, Any] | None = None) -> None:
    """Append a durable progress event. Call from workflow code (replay-safe)."""
    DBOS.write_stream(EVENTS_KEY, {"kind": kind, "data": data or {}})


def close_events() -> None:
    """Mark the event stream complete so readers (SSE clients) terminate."""
    DBOS.close_stream(EVENTS_KEY)


def read_events(run_id: str, after: int = -1) -> Iterator[tuple[int, dict[str, Any]]]:
    """Yield `(offset, event)` for every event with offset > `after`, until the stream closes."""
    offset = after + 1
    for event in DBOS.read_stream(run_id, EVENTS_KEY, offset=offset):
        yield offset, event
        offset += 1


async def aread_events(run_id: str, after: int = -1) -> AsyncIterator[tuple[int, dict[str, Any]]]:
    offset = after + 1
    async for event in DBOS.read_stream_async(run_id, EVENTS_KEY, offset=offset):
        yield offset, event
        offset += 1


class TokenBus:
    """In-memory fan-out of model tokens, keyed by run id. Not durable, not resumable."""

    def __init__(self) -> None:
        self._subscribers: dict[str, list[queue.SimpleQueue[str]]] = {}
        self._lock = threading.Lock()

    def subscribe(self, run_id: str) -> queue.SimpleQueue[str]:
        q: queue.SimpleQueue[str] = queue.SimpleQueue()
        with self._lock:
            self._subscribers.setdefault(run_id, []).append(q)
        return q

    def unsubscribe(self, run_id: str, q: queue.SimpleQueue[str]) -> None:
        with self._lock:
            subs = self._subscribers.get(run_id, [])
            if q in subs:
                subs.remove(q)
            if not subs:
                self._subscribers.pop(run_id, None)

    def publish(self, run_id: str, token: str) -> None:
        with self._lock:
            subs = list(self._subscribers.get(run_id, ()))
        for q in subs:
            q.put(token)


tokens = TokenBus()
