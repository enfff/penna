# ADR 0004: Filename-Derived Entry IDs With Minute-Slot Collisions

- Status: Accepted
- Date: 2026-08-24

## Context

Every entry needs a stable identifier. Penna has no central registry or
server; the journal is a plain git repository of Markdown files. The engine
currently derives identity from the filesystem: an entry file is named
`YYYYMMDDHHmm.md` and its id is the filename without extension.

One rule is specified in `docs/ENGINE_API.md` but decided nowhere:
"Engine resolves same-minute collisions by moving to next available minute
slot." Journaling can easily create two entries within one minute, so this
rule is load-bearing for uniqueness. It must be recorded and bounded before
frontends rely on it.

Related constraint from `docs/DATA_MODEL.md`: `created_at` / `updated_at`
are not yet durably stored, so the filename is also the only durable record
of creation time.

## Decision

1. Entry id equals the filename stem `YYYYMMDDHHmm`. No other id source.
2. On create, the engine formats the current UTC time as `YYYYMMDDHHmm`.
3. If that filename already exists, the engine increments the minute part
   until it finds a free slot. It never overwrites.
4. Ids are immutable after create. Update preserves the file name.
5. Deleting an entry frees its slot. Slot reuse after deletion is allowed.
6. The minute-shifted filename remains the displayable creation timestamp
   until durable timestamps land in a future model version.

## Alternatives Considered

- **UUIDs / random suffixes** (`202608241030-a7f3.md`). Globally unique with
  zero collision logic, but filenames stop being human-sortable and stop
  meaning anything in a file manager.
- **Millisecond filenames** (`YYYYMMDDHHmmss.SSS`). Pushes collisions out but
  not away; still needs fallback logic, and names get noisy.
- **Sequence numbers** (`000123.md`). Simple uniqueness, but loses all time
  information and invites renumbering temptation.
- **Central index file mapping ids to files.** One more artifact to merge,
  corrupt, and repair; violates the "filesystem is the registry" simplicity.

## Consequences

### Positive

- Directory listings are chronological without any metadata.
- No registry, no counter state, nothing to corrupt beyond the files.
- Ids stay stable across clones, machines, and git history.
- Collision handling is deterministic and testable.

### Negative

- A shifted slot means the filename no longer matches wall-clock time.
- Clock skew between machines yields valid but non-monotonic ids after sync;
  harmless because ordering uses lexicographic filename order only where
  explicitly chosen.
- Throughput is one entry per minute-slot; bursts shift minutes forward,
  so ids can run ahead of real time.
