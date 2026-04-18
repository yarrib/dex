Scaffold a new tool for the agent.

Tools live under `src/<agent>/tools/` and are auto-discovered by
`discover_tools()` in `tools/__init__.py`. Any public function with a docstring
becomes a tool.

Steps:

1. Create a new file: `src/<agent>/tools/<tool_name>.py`.

2. Implement the tool as a plain function returning a `ToolResult`:
   ```python
   from . import ToolResult


   def my_tool(arg: str) -> ToolResult:
       """One-line description — this becomes the tool's spec for the model."""
       try:
           result = do_work(arg)
           return ToolResult(success=True, data=result)
       except Exception as e:
           return ToolResult(success=False, data=None, error=str(e))
   ```

3. Write at least one test in `tests/test_<tool_name>.py` covering the happy
   path and one failure mode.

4. Add an eval case in `evals/cases/` that exercises the new tool end-to-end
   through the agent (not just the function in isolation).

5. Run `/test` and `/eval` to verify nothing regressed.

Do not register the tool manually — discovery is automatic. Do not add tools
that require side-effects in their docstring (e.g. "always call this first")
— that belongs in `prompts/system.md`.
