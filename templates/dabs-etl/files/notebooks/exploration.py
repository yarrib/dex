# Databricks notebook source
# MAGIC %md
# MAGIC # Exploration Notebook
# MAGIC
# MAGIC Use this notebook for interactive exploration and prototyping.

# COMMAND ----------

# MAGIC %md
# MAGIC ## Setup

# COMMAND ----------

catalog = "dev"
schema = "default"

spark.sql(f"USE CATALOG {catalog}")
spark.sql(f"USE SCHEMA {schema}")

# COMMAND ----------

# MAGIC %md
# MAGIC ## Explore Pipeline Output

# COMMAND ----------

# Read from the DLT-managed tables
# df = spark.table(f"{catalog}.{schema}.cleaned_<project>")
# display(df)
