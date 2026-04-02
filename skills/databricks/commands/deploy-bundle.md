Deploy a Databricks Asset Bundle to the configured workspace.

```bash
# Deploy to default target
databricks bundle deploy

# Deploy to a specific target
databricks bundle deploy --target staging
databricks bundle deploy --target production

# Deploy with dex pass-through (if configured)
dex db bundle deploy --target staging
```

Before deploying:
1. Validate the bundle configuration: `databricks bundle validate`
2. Confirm the target workspace URL in `databricks.yml`
3. Check that you have deploy permissions in the target workspace

After deploying:
- Verify resources were created: `databricks bundle summary`
- Run the workflow to confirm it executes: `databricks bundle run <job-name>`

Troubleshooting:
- `PERMISSION_DENIED` — check your workspace PAT or OAuth credentials
- `RESOURCE_CONFLICT` — another bundle with the same name is deployed; use a unique `bundle.name`
- Schema validation errors — run `databricks bundle validate` and fix reported fields
