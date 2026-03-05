# Databricks notebook source
# MAGIC %md
# MAGIC # Agent Evaluation Notebook
# MAGIC
# MAGIC Use this notebook to interactively evaluate your agent before deployment.

# COMMAND ----------

# MAGIC %md
# MAGIC ## Setup

# COMMAND ----------

# %pip install -q mlflow langchain langchain-databricks
# dbutils.library.restartPython()

# COMMAND ----------

import mlflow

catalog = "dev"
schema = "default"
experiment_name = f"/Shared/{{ project_name }}_evals"

mlflow.set_experiment(experiment_name)

# COMMAND ----------

# MAGIC %md
# MAGIC ## Load Agent

# COMMAND ----------

from {{ project_name }}.agent import build_agent

agent = build_agent()

# COMMAND ----------

# MAGIC %md
# MAGIC ## Run Evaluation Cases

# COMMAND ----------

test_cases = [
    {
        "input": "TODO: add your test question here",
        "expected": "TODO: add expected behavior",
    },
]

with mlflow.start_run(run_name="notebook_eval"):
    for i, case in enumerate(test_cases):
        result = agent.invoke({"input": case["input"]})
        print(f"Case {i}: {result['output']}")
        # TODO: score result against expected and log metric
        # mlflow.log_metric(f"case_{i}_pass", score)
