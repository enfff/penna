# ADR 0005: In-Process Engine Binding For First-Party Frontends

- Status: Accepted
- Date: 2026-08-24

## Context

No frontend exists yet (`cli/`, `tui/`, `gui/` directories are empty).
`docs/ENGINE_API.md` names three possible transports without choosing one:
"IPC/RPC/direct Rust binding". This blocks all frontend work because the
binding model determines packaging, error propagation, and async boundaries.

The engine API is already shaped like a message contract: methods take and
return serializable structs keyed by `session_id`.

## Decision

1. First-party frontends (CLI, TUI, GUI) link `penna-engine` as an
   in-process Rust dependency and call engine methods directly.
2. The method signatures and types in `docs/ENGINE_API.md` remain the logical
   contract. JSON shapes document the wire format for any future bridge;
   they do not require a bridge today.
3. No IPC/RPC layer gets built until a concrete second consumer exists that
   cannot link Rust: a remote client, a web frontend, or a third-party
   plugin host.
4. If that day comes, the bridge wraps the same engine API. It must not fork
   logic or bypass sessions.

## Alternatives Considered

- **Engine daemon with JSON-RPC over socket/stdio.** Language-agnostic and
  matches the JSON docs literally. Rejected for now: every frontend pays
  serialization, lifecycle management (spawn/health/restart), and version
  skew between engine and frontend before one line of UI exists.
- **C ABI / FFI library.** Only pays off for non-Rust hosts. Adds unsafe
  boundary maintenance with no current consumer.
- **HTTP REST service.** Worst fit: network semantics (ports, auth, latency)
  for a local-first single-user app.

## Consequences

### Positive

- CLI/TUI work starts immediately; zero glue code.
- Errors propagate as typed `EngineError` results, no encoding layer.
- One versioned artifact per release (ADR 0002 covers all crates).

### Negative

- Non-Rust frontends need the future bridge; nothing reusable ships now.
- A GUI crash takes the engine down with it (acceptable: single-user,
  local-first; git keeps data safe).
