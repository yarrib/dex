Run a Databricks job or workflow defined in a bundle.

```bash
# Run a job defined in databricks.yml
databricks bundle run <job-name>

# Run with parameters
databricks bundle run <job-name> --python-params '["--date", "2024-01-01"]'

# Run with dex pass-through (if configured)
dex db bundle run <job-name>
```

To list available jobs in the bundle:
```bash
databricks bundle validate | grep -A5 "jobs:"
```

To monitor a running job:
```bash
databricks jobs get-run <run-id>
```

Troubleshooting:
- Job fails immediately — check cluster configuration and library dependencies
- `CLUSTER_TERMINATED` — cluster hit timeout or was manually stopped; check cluster logs
- Python import errors — ensure the bundle deploys the wheel/egg before the job runs
- Data access errors — verify Unity Catalog grants for the service principal running the job
