# ADR 0007: Timestamps Derived From Git History

- Status: Accepted
- Date: 2026-08-24

## Context

`Entry.created_at` and `updated_at` exist only at runtime
(`core/domain/entry.rs`). `docs/DATA_MODEL.md` states they are "not durably
stored in markdown file format yet". Consequence today: cloning or syncing a
journal to another machine yields entries whose timestamps the engine cannot
reconstruct except from file mtimes, which git does not preserve.

The filename encodes creation minute (ADR 0004), but update time has no home,
and backdating edits would require renaming files, which ADR 0004 forbids.

## Decision

1. The engine derives both timestamps from git history per entry path:
   - `created_at`: author date of the earliest commit touching `<id>.md`.
   - `updated_at`: author date of the latest commit touching `<id>.md`.
2. For uncommitted working-tree changes, `updated_at` is the current time
   and the entry is marked dirty; history remains authoritative once
   committed.
3. Before an entry's first commit (brand-new entry), both fields are the
   creation moment held in memory.
4. No new persisted schema: sidecars keep storing tags only. Timestamps are
   always recomputable, so nothing can be lost by deleting `.penna/`.
5. Filenames remain untouched by this decision; ADR 0004 immutability holds.

## Alternatives Considered

- **Timestamps in the sidecar JSON** (`{"tags": [...], "created_at": ...}`).
  Simple read path, but makes sidecar data load-bearing for core facts,
  contradicting its disposable-metadata role.
- **YAML frontmatter timestamps.** Deferred to v2 by DATA_MODEL; also puts
  metadata inside the portable body file we keep clean.
- **File mtime fallback everywhere.** Not preserved by git checkout/clone;
  already known-unreliable.

## Consequences

### Positive

- Timestamps survive clone, pull, and `.penna/` deletion with zero storage.
- History-backed values are tamper-evident via commit hashes.
- Amends/rebases recompute correctly on next read; no stale caches.

### Negative

- Reads need a log walk per entry path; list operations must batch or cache
  per session to stay fast on large journals.
- Rewriting history changes visible timestamps (acceptable: journals rarely
  rewrite; git reflog still shows originals).
