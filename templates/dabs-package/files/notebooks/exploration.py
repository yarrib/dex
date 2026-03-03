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
# MAGIC ## Explore
