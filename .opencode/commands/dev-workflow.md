---
description: Full dev workflow — ADR check, branch, slice implementation, gate, commit
---
Execute the full workflow defined in @.github/prompts/dev-workflow.prompt.md:

1. Confirm ADR coverage first; stop and draft one via /adr if missing.
2. Create branch `feat/NNNN-short-title`.
3. Implement in small vertical slices: domain → application → ports → adapters → engine API.
4. After each slice run the verification gate: `cargo test --workspace && cargo clippy --workspace`.
5. Run /arch-check before committing.
6. Stage only related files and commit with a conventional message.

$ARGUMENTS
