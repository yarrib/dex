Run the agent's eval suite.

Locate the eval runner (typically `evals/runner.py`) and execute it:

```bash
uv run python evals/runner.py
```

The runner iterates over `evals/cases/*.json`, invokes the agent, and prints a
pass/fail summary per case.

When reporting results:
1. State the pass/fail counts first.
2. For each failure, show the case name, expected behavior, and actual output.
3. If a case fails for a novel reason, suggest whether it indicates a prompt
   issue, a tool issue, or a missing eval case — do not rewrite prompts
   proactively; flag it and wait for direction.

If the runner errors out (e.g. missing API key, import error), treat that as a
failed run and report the error verbatim instead of inventing results.
