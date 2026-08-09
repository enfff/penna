# Architecture Boundary Check Prompt

Review the current git diff against docs/ARCHITECTURE.md and docs/DATA_MODEL.md.

## Rules to enforce

- core/domain and core/application must not import adapters/, engine/, cli/, tui/, gui/.
- core/domain and core/application must not use I/O or git2 directly.
- Any FS/git/OS interaction from core/application must go through core/ports traits.
- Concrete I/O crates belong only in adapters/.
- Frontend crates (cli/, tui/, gui/) should depend on engine/ API, not core/ or adapters/ directly.
- engine/ must not depend on frontend crates.

## Output format

If violations exist, repeat:

```markdown
## Violation N
- File / Location: path:line
- Rule broken: <quoted rule>
- Fix suggestion: <concrete fix>
```

If none:

```markdown
## No architecture violations found
```
