# Databricks notebook source
# MAGIC %md
# MAGIC # Exploration Notebook
# MAGIC
# MAGIC Use this notebook for interactive data exploration before training.

# COMMAND ----------

# MAGIC %md
# MAGIC ## Setup

# COMMAND ----------

catalog = "dev"
schema = "default"
experiment_name = "/Shared/{{ project_name }}_exploration"

spark.sql(f"USE CATALOG {catalog}")
spark.sql(f"USE SCHEMA {schema}")

import mlflow
mlflow.set_experiment(experiment_name)

# COMMAND ----------

# MAGIC %md
# MAGIC ## Load Data

# COMMAND ----------

# df = spark.table("your_feature_table")
# display(df)

# COMMAND ----------

# MAGIC %md
# MAGIC ## Explore

# COMMAND ----------

# TODO: add your exploratory analysis here
