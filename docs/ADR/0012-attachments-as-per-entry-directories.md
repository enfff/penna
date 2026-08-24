# ADR 0012: Attachments As Per-Entry Directories

- Status: Accepted
- Date: 2026-08-24

## Context

`docs/ENGINE_SCOPE.md` names attachments among the domain rules the engine
must enforce, but no model exists: `docs/DATA_MODEL.md` defers rich
metadata, and nothing in the tree stores binary content. Real journals
accumulate photos, scans, and audio; deciding now prevents schema churn
after frontends exist.

Constraints inherited from the data model: the repository must stay plain
git, human-browsable, and portable without Penna installed; export strips
`.penna/` sidecars by default.

## Decision

1. Attachments live in a per-entry directory next to the entry file:
   `journal-root/<entry_id>/` (e.g. `journal-root/202608241030/photo.jpg`).
   Pairing directory with file makes ownership obvious at a glance.
2. Entries reference attachments with relative Markdown links
   (`![photo](202608241030/photo.jpg)`). Files render on GitHub, in any
   Markdown viewer, and after export with zero tooling.
3. The sidecar `.penna/<id>.json` gains an optional additive field:
   `"attachments": [{"name": "photo.jpg", "bytes": 81234}]` — a manifest for
   fast listing. Readers ignore unknown fields, so v1 sidecars remain valid.
4. Planned engine API additions: `add_attachment`, `remove_attachment`,
   `list_attachments(session_id, entry_id)`. Frontends never write into the
   journal directory themselves.
5. `delete_entry` removes `<id>.md`, `<id>/`, and refreshes the sidecar —
   all three or nothing.
6. Binary files are plain git blobs. No size limit is enforced in v1 beyond
   a sanity cap constant; git handles the rest.
7. Git LFS is explicitly out of scope until a user actually hits repo-size
   pain; adopting it later only affects new files.

## Architecture Boundary

Attachment storage is exposed through the `AttachmentStore` port in
`core/ports` and implemented by `adapters/git`: the files are plain git
blobs that must join journal history, so they live beside entry I/O in the
git adapter (which already performs working-tree writes). Core/application
gains use cases; the engine exposes them per the standard layering. No
frontend touches `<entry_id>/` directories directly.

## Alternatives Considered

- **Git LFS for binaries.** Keeps checkouts lean but requires LFS on every
  machine that touches the repo and breaks the "plain git works anywhere"
  guarantee. Revisit on evidence, not speculation.
- **Central hashed store** (`attachments/ab/cd/abcd1234`). Deduplicates and
  keeps the root tidy, but severs the visible link between entry and file
  and needs a mapping layer to survive renames (ids are immutable anyway).
- **Defer entirely to v2.** Leaves ENGINE_SCOPE's promise unimplemented and
  risks frontends improvising incompatible storage.

## Consequences

### Positive

- Repository remains fully self-describing and browsable without Penna.
- Export portability holds: relative links survive sidecar stripping.
- Delete semantics stay atomic from the user's point of view.

### Negative

- Journal root accumulates one directory per attachment-bearing entry;
  acceptable clutter versus indirection.
- Large media bloats clones for everyone; mitigated by the LFS escape hatch
  recorded above.
