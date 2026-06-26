---
name: new-user
description: Help a new user get from zero to hero with dex
---

Help a new user get from zero to hero with dex. This skill has two modes — pick based
on what's asked:

- **Onboard me** — guide a real user to their first success (default).
- **Audit the onboarding** — play a brand-new user with zero prior knowledge and report
  where the docs/help leave them stuck.

dex is **100% Rust, a single binary** — no Python runtime required to build or run it.

---

## Mode 1 — Onboard me

**First, ask:** What is your role?

- Data Engineer (pipelines, ETL, batch jobs)
- Data Scientist (notebooks, experiments, model training)
- ML Engineer (serving, feature engineering, MLOps)
- Software Engineer (building or extending dex itself)

Then tailor the path to their role.

**Universal first steps (all roles):**

1. Install Rust (stable): `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
2. Build & install dex:
   ```bash
   git clone <repo> && cd dex
   cargo build --release          # binary at target/release/dex
   cargo install --path crates/dex-cli   # or add target/release to PATH
   ```
3. Verify: `dex --help` then `dex init --help`
4. Scaffold your first project:
   ```bash
   dex init --template python-package --dir ~/projects/my-first-project
   cd ~/projects/my-first-project
   ```

**Role-specific paths:**

- **Data/ML roles:** `dex init` to scaffold, then `dex agent new` for AI workflows. You
  won't touch dex's Rust code. See `docs/SPEC.md` for the full command reference, and
  `dex skills init` to install AI skills into your project.
- **Software Engineer (extending dex):** read `CLAUDE.md` and `docs/ARCHITECTURE.md` for
  the `dex-core` (logic) / `dex-cli` (UI) split. Run `cargo test` to verify your
  environment. Core logic goes in `crates/dex-core/src/`; CLI wiring in
  `crates/dex-cli/src/commands/`.

---

## Mode 2 — Audit the onboarding

You are a brand-new user. You have never seen this codebase and do not know Rust, cargo,
or any project conventions. **You can only do what the README, docs, `CLAUDE.md`, or
`--help` explicitly tell you to do** — never assume, infer from code, or rely on insider
knowledge.

Walk the real entry points in order — `README.md`, `dex --help`, any quickstart, then
`docs/`. Follow each documented step literally and record: what the doc said, what you
did, what happened, what was missing or unclear. Flag every unexplained term or
prerequisite (e.g. "install cargo" without saying how, jargon like "DABS" or "template
manifest").

Report findings by severity:
- 🔴 **Blocker** — cannot proceed; a step is missing, broken, or needs an undocumented prerequisite.
- 🟡 **Friction** — can proceed but will be confused or likely guess wrong.
- 🟢 **Polish** — minor clarity improvement.

End with **What worked well** and a prioritized list of the most impactful fixes. Every
gap you find is a real user saved from frustration.
