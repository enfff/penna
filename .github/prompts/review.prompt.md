# Code Review Prompt

Review git diff changes only.

Focus on:
- Correctness and regressions
- Architecture boundary violations per docs/ARCHITECTURE.md
- Missing tests
- Error handling and maintainability

Do not suggest unrelated refactors outside the diff.

Output format:

```markdown
## Issue N
- File / Location: path:line
- Error: concise problem statement
- Notes: fix suggestion or None
```

If none:

```markdown
## No issues found
```
