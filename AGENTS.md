# AGENTS.md

## Purpose

This file gives rules to the AI agent. The agent must read this file before it writes any code.

Primary custom Copilot agent for this repository:

- `.github/agents/Penna Developer.agent.md`

## Required Reading

The agent must read these documents in order, before any code change:

1. `docs/ENGINE_SCOPE.md`. This document defines what engine is responsible for and what it must never own.
2. `docs/ARCHITECTURE.md`. This document gives the layer rules and the dependency rule. It defines the engine/frontend separation.
3. `docs/DATA_MODEL.md`. This document gives the `Entry` structure, the JSON sidecar format, and the frontmatter schema.
4. All files in `docs/ADR/`. These files are the Architecture Decision Records. (Note: ADRs are being rewritten to reflect the new engine/frontend architecture.)

The agent must check the existing ADRs before it proposes a decision. The agent must not propose a decision that an ADR already makes. These decisions include:

- The engine uses Rust. Frontends are separate binaries (CLI, TUI, GUI).
- The document model is a TipTap JSON and Markdown hybrid.
- Version control and merges use `git2` and `libgit2`.
- The engine is completely independent of all frontends.

## Hard Constraints

- `core/domain` must not import code from `adapters/`, `engine/`, `cli/`, `tui/`, or `gui/`.
- `core/application` must not import code from `adapters/`, `engine/`, `cli/`, `tui/`, or `gui/`.
- `core/domain` must not contain I/O code.
- `core/domain` must not import `git2`.
- `core/domain` must not import any frontend crate (CLI, TUI, GUI).
- Every new use case in `core/application` must have a matching unit test in `core/tests`.
- Frontends must not access git directly.
- Frontends must not access the filesystem directly.
- Frontends must call the engine API for any git or filesystem operation.
- The engine API must call a use case in `core::application`. The engine must not contain frontend-specific logic.
- The Markdown importer must not fail on unknown syntax.
- The Markdown importer must degrade gracefully on unknown syntax. The importer must fall back to plain text or a raw block.
- The Markdown exporter must remove the JSON sidecar by default. This keeps exported files portable.

## Where New Code Goes

| Task | Location |
|------|----------|
| New use case | `core/application` |
| New I/O adapter | `adapters/`, with a matching port trait in `core/ports` |
| New engine API surface | `engine/` |
| New CLI feature | `cli/`, calling engine API |
| New TUI feature | `tui/`, calling engine API |
| New GUI feature | `gui/`, calling engine API |

## Prompt Index

The agent can use the GitHub Copilot prompts in `.github/prompts/`.

Legacy aliases are also preserved in `.opencode/command/` and `.opencode/skill/`.
These files map to the Copilot prompts so existing muscle-memory commands still work.

| Prompt | Purpose |
|--------|---------|
| `review.prompt.md` | Runs an AI code review of the current git diff. |
| `arch-check.prompt.md` | Checks the diff for layer violations against `docs/ARCHITECTURE.md`. |
| `test-gen.prompt.md` | Generates unit tests, including Markdown import and export round-trip fixtures. |
| `adr.prompt.md` | Drafts a new ADR file from a decision discussed in the session. |
| `commit.prompt.md` | Generates a conventional commit message from the staged diff. |

## When in Doubt

1. Check the existing ADRs for a matching decision.
2. If no ADR covers the decision, draft a new ADR. Use the `adr.prompt.md` prompt for this step.
3. Do not deviate from an existing ADR. Do not deviate silently.
4. If a constraint in this file blocks a needed change, stop. Ask the user before you proceed.
5. Remember: The engine is frontend-agnostic. Frontends are plug-in replacements.