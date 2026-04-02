Review a pull request.

Steps:
1. Understand what the PR is trying to accomplish (read the description and diff).
2. Check correctness: does the code do what it claims?
3. Check test coverage: are new behaviors tested?
4. Check for regressions: could this break existing behavior?
5. Check code style: does it match the project's conventions?
6. Leave actionable, specific comments — not vague suggestions.

Focus on:
- Logic errors and edge cases
- Missing error handling
- Security issues (injection, auth bypass, data leaks)
- Performance problems in hot paths
- API contract changes that aren't backwards-compatible

Do not nitpick formatting if the project has an auto-formatter — those issues will
be caught by CI. Focus on substance.
