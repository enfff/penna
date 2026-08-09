# Commit Message Prompt

Generate a Conventional Commits message from staged changes.

## Steps

1. Read staged diff with git diff --staged.
2. If empty, output exactly:
No staged changes found. Stage files with git add before running this prompt.
3. Choose type from: feat, fix, refactor, test, docs, chore, build, perf, style.
4. Infer scope from dominant path:
- core/domain -> domain
- core/application -> application
- core/ports -> ports
- adapters/git -> git
- adapters/fs -> fs
- adapters/markdown -> markdown
- engine -> engine
- cli -> cli
- tui -> tui
- gui -> gui
- docs/ADR -> adr
- docs -> docs
- .github/prompts -> prompts
5. Output format:

```text
type(scope): short imperative summary under 72 chars

- Bullet point for non-trivial detail.
- Optional second bullet.
```

Use direct wording. No vague summaries.
