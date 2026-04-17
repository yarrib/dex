Iterate on the agent's system prompt using evidence, not vibes.

The system prompt lives at `src/<agent>/prompts/system.md`. Changes to it must
be justified by an eval result — not by a single failure observed in traces.

Workflow:

1. **Baseline**: run `/eval` and record the pass/fail breakdown. Save the
   current prompt (e.g. `git stash` or copy to `system.baseline.md`).

2. **Hypothesis**: state explicitly what you believe is wrong and what change
   you expect to improve. Example: "The agent ignores tool errors — adding an
   explicit 'check tool_result.success before continuing' instruction should
   fix cases 3 and 5."

3. **Minimal edit**: make the smallest possible change to `system.md`. Do not
   rewrite the whole prompt.

4. **Re-run evals**: run `/eval` again. Compare against baseline.
   - If the target cases pass and no regressions: keep the change.
   - If regressions appear: revert and form a new hypothesis.

5. **Document**: update `AGENTS.md` or a nearby comment if the change encodes
   a non-obvious constraint (e.g. a safety rule, a domain convention).

Do not change `prompts/system.md` without running evals before and after.
