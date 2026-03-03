You are helping a new user get from zero to hero with dex.

**First, ask:** What is your role?

- Data Engineer (building pipelines, ETL, batch jobs)
- Data Scientist (notebooks, experiments, model training)
- ML Engineer (model serving, feature engineering, MLOps)
- Software Engineer (building the CLI itself or extending it)

Then tailor the onboarding path to their role.

---

**Universal first steps (all roles):**

1. Install prerequisites:
   - Rust stable: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
   - uv: `curl -LsSf https://astral.sh/uv/install.sh | sh`
   - Databricks CLI: `curl -fsSL https://raw.githubusercontent.com/databricks/setup-cli/main/install.sh | sh`

2. Build dex:
   ```bash
   git clone <repo>
   cd dex
   make dev        # uv sync + maturin develop
   ```

3. Verify:
   ```bash
   dex --help
   dex init --help
   ```

4. Scaffold your first project:
   ```bash
   dex init --template default --dir ~/projects/my-first-project
   cd ~/projects/my-first-project
   uv sync
   ```

---

**Role-specific paths:**

**Data Engineer / Data Scientist / ML Engineer:**
→ Your goal is `dex init` to scaffold a project, then `dex agent new` for AI workflows.
→ You won't need to modify Rust code.
→ Check `docs/SPEC.md` for the full command reference.

**Software Engineer (extending dex):**
→ Read `CLAUDE.md` fully.
→ Read `docs/ARCHITECTURE.md` to understand the Rust/Python split.
→ Run `make test` to verify your environment.
→ Start with `python/dex/cli.py` for Python-side changes.
→ Start with `crates/dex-core/src/` for core logic changes.

---

**If anything is unclear or broken, flag it here.** The path to success should be obvious.
If it isn't, that's a documentation or UX bug worth reporting.
