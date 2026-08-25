# ENGINE_SCOPE.md

Purpose: define what Penna engine does, what it does not do, and how frontends must use it.

## Why Engine Exists

Git gives history and merge mechanics.
Penna engine gives journal-specific behavior, safety rules, and stable API over git/filesystem/markdown details.

## Engine Responsibilities

1. Enforce domain rules for entries, journals, tags, attachments, and merge-conflict states.
2. Execute application use cases (create/get/list/update/delete/import/export/sync orchestration).
3. Validate and preserve data contracts from `docs/DATA_MODEL.md`.
4. Normalize behavior across platforms and frontends (IDs, timestamps, naming, serialization).
5. Surface conflicts as explicit domain values and resolution workflows.
6. Route all I/O through ports and adapters, never directly from domain/application.
7. Expose stable engine API for CLI/TUI/GUI and future clients.

## Engine Non-Responsibilities

1. No frontend rendering, widget behavior, or view state.
2. No direct CLI/TUI/GUI assumptions in core logic.
3. No direct calls from frontends to git/filesystem/markdown adapters.
4. No silent data loss when parsing malformed markdown/frontmatter/sidecar.

## Contract With Frontends

1. Frontends send intent to engine API.
2. Engine returns structured success/errors/conflicts.
3. Frontends own interaction UX; engine owns correctness and deterministic outcomes.

### Engine API Surface (v1 — shipped)

1. `connect_journal(repo_path)` -> open/init repository and return session handle.
2. `journal_status(session_id)` -> branch/head/dirty + merge state metadata.
3. `disconnect_journal(session_id)` -> close session handle.
4. `list_entries(session_id)` -> list entry models (working-tree first).
5. `get_entry(session_id, id)` -> load single entry.
6. `create_entry(session_id, request)` -> create plain Markdown entry using `YYYYMMDDHHmm.md` id/filename pattern.
7. `update_entry(session_id, request)` -> update existing entry while preserving `created_at`.
8. `delete_entry(session_id, id)` -> delete entry (plus attachments and sidecar).
9. `sync_journal(session_id)` -> fetch/merge/push journal state; auto-concludes merges.
10. `clone_journal(request)` -> clone remote repo and open session.
11. `resolve_journal_path(session_id)` -> return canonical local repository path.
12. `pull_journal(session_id)` and `push_journal(session_id)` -> explicit directional sync controls.

### Engine API Surface (v2 additions)

- Conflict flow (ADR 0014): `get_entry_conflict`, `resolve_entry_conflict`, `reconcile_journal`.
- Attachments (ADR 0012): `add_attachment`, `get_attachment`, `list_attachments`, `remove_attachment`.
- Errors are a closed five-code set (ADR 0009); credentials resolve
  provider-neutrally (ADR 0010). See `docs/ENGINE_API.md` for the full
  contract including the threading rules.

## Source Of Truth Hierarchy

1. `docs/ENGINE_SCOPE.md` for mission and scope.
2. `docs/ENGINE_API.md` for frontend integration contract.
3. `docs/ARCHITECTURE.md` for layering and dependency boundaries.
4. `docs/DATA_MODEL.md` for storage and conversion contracts.
5. `docs/ADR/` for historical architecture decisions.

## Near-Term Engine Milestones

1. Entry lifecycle v1: create/get/list/update/delete with strict invariants and tests.
2. Plain Markdown body storage plus `.penna/<id>.json` tags sidecar contract.
3. Sync API for pull/push or unified journal sync with structured status reporting.
4. Engine API surface that frontends can consume without touching adapters directly.
5. Remote onboarding API (`clone_journal`) with explicit path and sync controls.