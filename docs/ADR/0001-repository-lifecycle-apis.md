# ADR 0001: Repository Lifecycle APIs

- Status: Proposed
- Date: 2026-08-13

## Context

Frontend needs explicit repo lifecycle workflow:

1. Download/clone journal repository from remote.
2. Determine canonical local path in OS.
3. Update local repo from remote (pull).
4. Publish local changes to remote (push).

Current engine already supports:

- `connect_journal(repo_path)` for local open/init.
- `journal_status(session_id)` with `repo_path`.
- `sync_journal(session_id)` unified smart sync.

Gap: no explicit clone API, no explicit pull/push directional APIs.

## Decision

Introduce planned engine API surface for repository lifecycle:

1. `clone_journal(request)`
2. `resolve_journal_path(session_id)`
3. `pull_journal(session_id)`
4. `push_journal(session_id)`

Request/response contracts documented in:

- `docs/ENGINE_API.md`
- `docs/ENGINE_API_COPYPASTE.md`

## API Semantics

### clone_journal

- Clones remote repository into user-chosen local directory.
- Opens repository as engine session.
- Returns same `JournalSession` shape as `connect_journal`.

### resolve_journal_path

- Returns canonical absolute path for active session.
- No filesystem mutation.

### pull_journal

- Pull-only direction.
- Never pushes local commits.
- Returns `SyncReport` statuses: `up_to_date`, `pulled`, `no_remote`, `no_branch`, `diverged`.

### push_journal

- Push-only direction.
- Never fetches+applies remote commits.
- Returns `SyncReport` statuses: `up_to_date`, `pushed`, `no_remote`, `no_branch`, `diverged`.

## Architecture Constraints

1. Engine API remains frontend-agnostic.
2. Use cases live in `core/application`.
3. Port traits defined in `core/ports` before adapter changes.
4. Git I/O only in `adapters/git`.
5. Frontends must not call adapters directly.

## Consequences

Positive:

1. Frontend gets beginner-friendly explicit actions (clone/path/pull/push).
2. Better UX control than one smart sync button.
3. Backward-compatible with existing `sync_journal`.

Tradeoffs:

1. More API surface to maintain.
2. Need clear docs to avoid confusion between `sync_journal` and directional APIs.

## Alternatives Considered

1. Keep only `sync_journal`.
   - Rejected for frontend UX clarity; hard to map explicit pull/push buttons.
2. Do clone in frontend and only connect in engine.
   - Rejected by architecture rule; frontend should not call git directly.

## Rollout Plan

1. Add new use cases in `core/application` with tests in `core/tests`.
2. Add/extend sync port traits in `core/ports` for directional operations.
3. Implement in `adapters/git` and add adapter integration tests.
4. Wire in `engine` and add engine API tests.
5. Keep `sync_journal` as convenience API.
