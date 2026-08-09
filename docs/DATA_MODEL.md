# DATA_MODEL.md

Status: v1 storage contract.
Scope: entry storage, file naming, and current persistence rules.

This document defines what Penna persists today.
It favors the real v1 product loop over a richer future model.

## Core Data Principle

Plain Markdown is the durable source of truth.
For v1, each entry is a single Markdown file that remains human-readable without Penna.

## Journal Layout

A journal is a git repository with Markdown entries at the repo root.

```text
journal-root/
├── .git/
├── YYYYMMDDHHmm.md
├── YYYYMMDDHHmm.md
└── ...
```

Rules:

1. Entry filename format: `YYYYMMDDHHmm.md`.
2. Entry id is the filename without `.md`.
3. Files are plain Markdown only.
4. No required frontmatter in v1.
5. No required sidecar files in v1.

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

## Current Persisted Model

The engine `Entry` model currently includes:

- `id`
- `title`
- `body`
- `tags`
- `created_at`
- `updated_at`

Persistence rules for v1:

1. `id` persists via filename.
2. `title` persists via the first heading in the Markdown body.
3. `body` persists as Markdown text.
4. `tags` are not durably stored in the file format yet.
5. `created_at` and `updated_at` are not durably stored in the file format yet.

This is intentional for v1 simplicity.

## Deferred Metadata Model

The richer metadata model is deferred.
This includes:

1. YAML frontmatter persistence.
2. Unknown frontmatter preservation.
3. `penna_sidecar` file references.
4. Sidecar file lifecycle.
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
