# ADR 0009: Five-Code Public Error Taxonomy

- Status: Accepted
- Date: 2026-08-24

## Context

The engine returns typed errors internally (`EngineError`), and
`docs/ENGINE_API.md` defines a bridge shape
`{ "code": "...", "message": "..." }` with exactly five codes:
`NOT_CONNECTED | IO | REPO | VALIDATION | CONFLICT`. The mapping rules are
unwritten: which internal variants map to which code, whether new codes may
appear, and what stability frontends can rely on. Without a recorded rule,
each future bridge (ADR 0005 keeps that door open) invents its own mapping.

## Decision

1. The five public codes are the stable, versioned error API:
   - `NOT_CONNECTED`: unknown, stale, or disconnected `session_id`.
   - `IO`: filesystem failures outside git (permissions, missing dirs).
   - `REPO`: git/repository failures (corrupt repo, network sync errors).
   - `VALIDATION`: rejected input (bad id format, empty title, bad request).
   - `CONFLICT`: diverged state requiring resolution (ADR 0006 values).
2. Every `EngineError` variant maps to exactly one code. The mapping table
   lives in the engine crate next to `EngineError` and is unit-tested.
3. Codes are closed: adding a sixth requires an ADR and a minor-version bump.
4. `message` strings are human-readable diagnostics, never parsed by
   frontends; only `code` is contractual.
5. Domain invariants surface as `VALIDATION`, never as panics or `unwrap`
   across the engine API boundary.

## Alternatives Considered

- **One code per error variant** (dozens of codes). Maximum precision for
  frontends, but freezes internal refactor granularity forever; every new
  failure mode becomes an API event.
- **Numeric codes.** Compact but unreadable in logs and docs.
- **No taxonomy; pass messages through.** Forces frontends to string-match,
  which breaks on every wording change.

## Consequences

### Positive

- Frontend switch statements stay five cases wide indefinitely.
- Mapping tests make accidental reclassification visible at CI time.
- ADR 0006's conflict values slot into `CONFLICT` without extension.

### Negative

- Some fidelity is lost inside a bucket (e.g. permission vs disk-full both
  map to `IO`); detailed handling stays in `message` and typed internals.
