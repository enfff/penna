# ADR 0003: Tags Persist In JSON Sidecars

- Status: Accepted
- Date: 2026-08-24

## Context

Entries need tags. The v1 data model keeps plain Markdown as the durable
source of truth for note text and defers rich metadata (frontmatter,
attachments, revisions). Tag management use cases (`list_tags`, `add_tag`,
`remove_tag`, `update_tag`) are implemented in `core/application` and exposed
through the engine API. Storage still needs a recorded decision because three
options were viable and consumers depend on the chosen layout.

Current implemented behavior:

1. Tags travel in `create_entry` / `update_entry` requests.
2. Tags persist per entry in `.penna/<entry_id>.json`.
3. A global tag catalog persists in `.penna/tags.json`.
4. Missing or malformed sidecar falls back to an empty tag list, non-fatal.
5. Deleting an entry removes its sidecar.

No ADR covers this today.

## Decision

Tags persist outside the Markdown file, in JSON sidecars:

1. Per-entry tags live in `.penna/<entry_id>.json` with shape
   `{ "tags": ["...", "..."] }`.
2. A journal-wide catalog lives in `.penna/tags.json`.
3. Entry Markdown files never carry Penna metadata in v1. Unknown
   frontmatter in imported files stays untouched as literal text.
4. A missing sidecar means zero tags. A malformed sidecar degrades to zero
   tags. Both are non-fatal.
5. Delete entry must remove the matching sidecar.
6. `remove_tag` / `update_tag` must keep the per-entry sidecars and the
   global catalog consistent.

## Alternatives Considered

- **YAML frontmatter inside the entry file.** One file per note, familiar
  from Obsidian/Jekyll. Rejected for v1: it couples note text with mutable
  metadata, complicates round-trip fidelity, and was explicitly deferred in
  `docs/DATA_MODEL.md`. May return as v2.
- **Single journal-wide tags file only.** One place to edit, but every entry
  read needs an index lookup and concurrent edits conflict more often in git.
- **Embedded database (e.g. SQLite) for tags.** Fast queries, but binary
  blobs break the "every persisted artifact is human-readable and
  git-mergeable" property of a journal repo.

## Consequences

### Positive

- Note bodies stay portable; any editor sees clean Markdown.
- Sidecars are small, diff-friendly JSON that git merges well.
- Corrupt sidecars never destroy note text (graceful degradation rule).
- Export can drop sidecars by default and remain lossless for text.

### Negative

- Two artifacts per tagged entry; create/update/delete must handle both.
- Catalog drift is possible; mitigated by
  `validate_sidecar_integrity` checks.
- Frontmatter-based interop with third-party tools waits for v2.
