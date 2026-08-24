# ADR 0006: Conflict Resolution Via In-App Merge Editor

- Status: Accepted
- Date: 2026-08-24

## Context

`docs/ENGINE_SCOPE.md` requires the engine to "surface conflicts as explicit
domain values and resolution workflows" while frontends own interaction UX.
Today the engine reports `diverged` with ahead/behind counts, but there is no
conflict domain type, no resolution use case, and no defined behavior when
the same entry is edited on two machines before syncing. Journals make this
common: yesterday's entry gets edited on a laptop and again on a desktop.

The user-facing model is decided: the user opens a merge editor in their app
and fixes the conflict there by hand.

## Decision

1. The engine never auto-resolves entry bodies. No last-writer-wins, no
   automatic textual merge.
2. When sync detects divergence affecting entries, the engine computes a
   three-way view per affected entry using libgit2 merge machinery:
   - `base`: common ancestor version,
   - `ours`: local working version,
   - `theirs`: remote version.
3. This surfaces as a domain value
   `EntryConflict { entry_id, base, ours, theirs }`, fetched with a use case
   such as `get_entry_conflict(session_id, entry_id)`.
4. The frontend renders its own merge editor from those three texts and the
   user produces the resolved body in-app.
5. A new use case
   `resolve_entry_conflict(session_id, entry_id, resolved_body)` accepts the
   user's merged text and writes it as a normal update commit. Git history
   preserves both conflicting sides plus the resolution.
6. Quick actions ("keep ours", "keep theirs") are optional frontend
   conveniences; they are presets of `resolved_body` and travel through the
   same use case. They add no engine surface.
7. Tag sidecars merge by set union without user interaction; tags are sets,
   so union is lossless. Only bodies need the editor.
8. `sync_journal` / `pull_journal` report `diverged` plus conflicted entry
   ids instead of failing as an all-or-nothing merge.

## Architecture Boundary

The merge editor is pure frontend. The engine supplies the three-way data
and accepts arbitrary resolved text; it must contain no widget, layout, or
interaction logic. Any frontend that cannot show a graphical editor (CLI)
satisfies this contract by presenting the three versions and accepting
edited input.

## Alternatives Considered

- **Automatic textual merge for bodies.** Prose merges yield grammatically
  valid nonsense users may not notice. Rejected.
- **Last-writer-wins by timestamp.** Silent data loss; violates the engine's
  no-silent-data-loss rule.
- **Duplicating both sides into two files** (`<id>-theirs.md`). Breaks the
  id-is-filename invariant and litters listings.
- **Engine-side conflict wizard flow.** Would put interaction UX inside the
  engine, violating the frontend contract.

## Consequences

### Positive

- Full user control over merged prose; nothing is silently lost.
- Engine stays UI-free; contract is one read + one write use case.
- Every frontend implements the same simple contract regardless of toolkit.

### Negative

- Each frontend must build merge-editor UI (three-pane diff at minimum).
- Long-lived divergence accumulates conflicts; mitigated because every sync
  attempt surfaces them immediately.
