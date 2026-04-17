Inspect the agent's recent MLflow traces.

MLflow traces capture the full tool-use trajectory of each agent run: prompts,
tool calls, tool results, and the final response.

Steps:

1. Open the MLflow UI pointed at this project's experiment:
   ```bash
   mlflow ui --backend-store-uri ./mlruns
   ```
   Or use the Databricks MLflow UI if the agent runs on Databricks.

2. Filter to the agent's experiment (named after `project_name` in
   `src/<agent>/agent.py`).

3. For each recent run, review:
   - **Input**: what the user / upstream system asked for.
   - **Tool calls**: which tools were invoked, with what arguments.
   - **Tool results**: success/error, latency.
   - **Final output**: the agent's response.

Flag any of:
- Tool calls that returned errors but the agent ignored them.
- Repeated tool calls that suggest the agent is looping.
- Final outputs that look like the `bad_output` described in `AGENTS.md`.
- Latency spikes or excessive token usage.

Report findings as a bulleted list with run IDs so the human can open each one.
Do not modify prompts or tools based on a single trace — flag the pattern and
wait for direction.
