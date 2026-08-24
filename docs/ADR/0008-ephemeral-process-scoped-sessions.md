# ADR 0008: Ephemeral Process-Scoped Sessions

- Status: Accepted
- Date: 2026-08-24

## Context

Engine sessions are an in-memory `Mutex<HashMap<String, SessionState>>` in
`engine/src/lib.rs`, keyed by `session-<nanosecond-timestamp>` strings.
Nothing documents their lifecycle contract: what happens on process restart,
whether the same repository may be opened twice, and whether disconnect has
data effects. Frontends cannot be built correctly against unspecified
semantics.

## Decision

1. Sessions are ephemeral and process-scoped. They exist only in engine
   memory; nothing about a session persists to disk beyond the underlying
   git state it manipulates.
2. A process restart invalidates all session ids. Reconnection is always
   `connect_journal(repo_path)` returning a fresh id. There is no
   resume/rehydrate path.
3. Multiple concurrent sessions on one repository are permitted within a
   single engine instance. Correctness of interleaved writes is guaranteed
   by git's object model plus the adapters' existing commit flow, not by
   session exclusivity.
4. `disconnect_journal(session_id)` only drops the handle. It never commits,
   reverts, or touches the working tree.
5. Unknown or stale session ids return the typed `NOT_CONNECTED` error code;
   frontends must treat it as "reconnect", not as failure of user intent.
6. Session ids stay opaque strings; clients must not parse them.

## Alternatives Considered

- **Persistent session registry** (`.penna/sessions.json`). Adds a mutable
  artifact for zero benefit: a reconnect is one idempotent call.
- **Single-session lock per repo** (second open returns error). Safer against
  accidental interleaving but blocks legitimate read-only inspection while a
  GUI holds the journal open.

## Consequences

### Positive

- Trivial mental model: connect → use → disconnect, repeat anywhere.
- No session garbage collection problem across restarts.
- Idempotent reconnect makes frontend crash recovery a one-liner.

### Negative

- Long-lived frontends must handle `NOT_CONNECTED` after any crash/restart.
- Concurrent sessions rely on adapter discipline rather than enforced
  exclusion; adapter tests must cover interleaved use cases.
