Create a well-formed git commit.

Follow these conventions:
1. Stage only the files relevant to the logical change.
2. Write a commit message with a conventional prefix:
   - `feat:` — new feature
   - `fix:` — bug fix
   - `refactor:` — restructuring without behavior change
   - `docs:` — documentation only
   - `test:` — tests only
   - `chore:` — build, tooling, CI
3. Use imperative mood: "add X" not "added X".
4. Keep the subject line under 72 characters.
5. If the change needs explanation, add a body after a blank line.

Example:
```
feat(auth): add OAuth2 token refresh flow

Tokens now refresh automatically 60 seconds before expiry.
Existing refresh_token() callers are not affected.
```

Before committing:
- Run tests: confirm nothing is broken.
- Run linters: no new warnings.
- Review the diff: no debug code, no secrets, no unrelated changes.
