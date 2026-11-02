---
name: durable-workflows
description: Replay-safety rules for DBOS durable workflows. Use when editing workflow or step code (workflows/, steps/), adding a tool, or changing the agent loop in this project.
---

# Durable workflows (DBOS)

A workflow is **re-run from the top** after a crash or redeploy. Steps that already finished
return their recorded output instead of executing. That only works if the workflow body is
deterministic and every effect lives in a step.

## Rules

1. **Workflow body = deterministic glue.** No clock, random, network, env or file I/O in
   `@DBOS.workflow` code. Read those inside a step (see `begin_run`, which freezes the prompt
   and settings for a run).
2. **Every non-deterministic call is a `@DBOS.step`** — model calls, tool calls, and also
   `datetime.now()` / `uuid4()` when the value affects control flow.
3. **Steps return JSON-serializable values** (`pydantic .model_dump()`, dicts, lists, str,
   numbers). Never return model instances, clients, or generators.
4. **Steps are at-least-once.** A crash after a side effect but before DBOS records the step
   re-runs it. Make writes idempotent with the key from `steps/tools.py:idempotency_key`
   (workflow id + step id + tool name): pass it to APIs that accept one, otherwise check for an
   existing result before writing (see `agent/tools/example_tool.py`).
5. **Changing step order is a breaking change.** Adding, removing or reordering steps in
   `agent_run` breaks recovery of in-flight runs. Bump `APP_VERSION`; DBOS recovers only runs
   from the same application version.
6. **Human waits use `DBOS.recv`** (see `steps/approval.py`), never `time.sleep` or polling.
   The wait survives restarts.
7. **Stream tokens live, checkpoint completions.** Tokens go to the in-memory bus; progress
   events use `emit()` (durable).

## Adding a tool

1. Write `fn(arguments, idempotency_key) -> dict` in `agent/tools/` — idempotent by key.
2. Register a `Tool` in `agent/tools/__init__.py` (set `needs_approval=True` for risky effects).
3. Add a test that runs the tool twice with the same key and asserts one effect
   (`tests/test_idempotency.py` shows how).
