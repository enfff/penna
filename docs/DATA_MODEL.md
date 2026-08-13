# DATA_MODEL.md

Status: v1 storage contract + planned tags sidecar contract.
Scope: entry storage, file naming, and tags metadata persistence path.

This document defines what Penna persists today.
It favors the real v1 product loop over a richer future model.

## Core Data Principle

Plain Markdown is durable source of truth for note body.
Tags metadata persists in optional sidecar JSON file.

## Journal Layout

A journal is a git repository with Markdown entries at the repo root.

```text
journal-root/
├── .git/
├── .penna/
│   ├── YYYYMMDDHHmm.json
│   ├── YYYYMMDDHHmm.json
│   └── ...
├── YYYYMMDDHHmm.md
├── YYYYMMDDHHmm.md
└── ...
```

Rules:

1. Entry filename format: `YYYYMMDDHHmm.md`.
2. Entry id is the filename without `.md`.
3. Files are plain Markdown only.
4. No required frontmatter in v1.
5. Tags sidecar path target: `.penna/<entry_id>.json`.

## Entry File Anatomy

Each entry file is plain Markdown.
The first Markdown heading is treated as the title when present.

```markdown
# First Day Back

Entry body in standard Markdown.
```

Fallback behavior:

1. If the first non-empty line starts with `# `, the title is the heading text.
2. Otherwise, the entry title falls back to `Untitled`.
3. The file body remains standard Markdown text.

## Current / Planned Persisted Model

The engine `Entry` model currently includes:

- `id`
- `title`
- `body`
- `tags`
- `created_at`
- `updated_at`

Persistence rules:

1. `id` persists via filename.
2. `title` persists via the first heading in the Markdown body.
3. `body` persists as Markdown text.
4. `tags` planned durable storage in `.penna/<id>.json` as JSON array.
5. `created_at` and `updated_at` are not durably stored in markdown file format yet.

Sidecar JSON v1 (minimal):

```json
{
	"tags": ["work", "idea"]
}
```

Behavior target:

1. Missing sidecar => tags default to empty list.
2. Malformed sidecar => non-fatal fallback to empty tags.
3. Delete entry removes both `.md` and matching `.penna/<id>.json`.

## Deferred Metadata Model

Richer metadata model beyond tags sidecar is deferred.
This includes:

1. YAML frontmatter persistence.
2. Unknown frontmatter preservation.
3. Rich sidecar fields (attachments/revisions/blocks).
4. Full sidecar integrity/repair workflows.
5. Rich block fidelity beyond plain Markdown.

These may return in a future version once the editor and sync flows require them.

## Import/Export Contracts

Importer and exporter behavior must remain best-effort and non-destructive.

1. Import must not hard-fail on unsupported Markdown syntax.
2. Unsupported constructs should fall back to plain text or raw block representation.
3. Export must produce valid plain Markdown.
4. Frontend-visible storage should stay human-readable.

## Invariants

1. Every persisted entry has a stable id derived from filename.
2. Removing Penna-specific logic must not destroy note text.
3. Entry files remain readable in any plain Markdown editor.
4. Create, update, and delete must affect both git history and working-tree files.

## Testing Requirements

At minimum, maintain tests for:

1. Create entry writes a real Markdown file.
2. Get entry reads title/body back from plain Markdown.
3. Update entry rewrites the Markdown file.
4. Delete entry removes the file.
5. Legacy plain Markdown parsing remains stable.

## Future Work

If richer metadata becomes necessary, introduce it explicitly as v2 rather than leaving v1 half-migrated.

## Related Documents

- [Architecture](./ARCHITECTURE.md)
- [ENGINE_SCOPE](./ENGINE_SCOPE.md)
