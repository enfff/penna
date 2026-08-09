# DATA_MODEL.md

Status: living specification.
Scope: entry storage, frontmatter, sidecar, and conversion contracts.

This document defines Penna's durable data contracts.
It complements [ARCHITECTURE.md](./ARCHITECTURE.md).

## Core Data Principle

Markdown is the durable source of truth.
The sidecar is supplementary metadata for fidelity that Markdown alone cannot preserve.
Deleting sidecars must never destroy textual content.

## Journal Layout

A journal is a git repository with human-readable Markdown entries.

```text
journal-root/
├── .git/
├── .penna/
│   ├── config.yaml
│   ├── index.sqlite
│   ├── sidecars/
│   │   └── YYYY-MM-DD-slug.json
│   └── attachments/
│       └── <entry-slug>/...
└── YYYY-MM-DD-slug.md
```

Rules:

1. Entry filename format: `YYYY-MM-DD-slug.md`.
2. Slug is derived from title for readability only.
3. Canonical identity is frontmatter `id`, not filename.
4. `.penna/index.sqlite` and sidecars are rebuildable caches.

## Entry File Anatomy

Each entry file has YAML frontmatter followed by Markdown body.

```markdown
---
id: 8f14e45f-ceea-467e-9c9a-3f6b30930f39
title: First Day Back
created_at: 2024-01-15T08:32:00-05:00
updated_at: 2024-01-15T21:14:03-05:00
tags:
  - work
  - reflection
mood: content
penna_sidecar: sidecars/2024-01-15-first-day-back.json
---

# First Day Back

Entry body in standard Markdown.
```

## Frontmatter Schema

| Field | Type | Required | Rule |
|---|---|---|---|
| `id` | UUID string | Yes | Immutable per entry |
| `title` | string | Yes | Human title and slug source |
| `created_at` | RFC 3339 datetime | Yes | Set at creation |
| `updated_at` | RFC 3339 datetime | Yes | Updated on mutation |
| `tags` | list of strings | No | Normalize to lowercase kebab-case in domain |
| `mood` | string | No | Optional mood marker |
| `penna_sidecar` | relative path string | No | Path under `.penna/sidecars/` |
| unknown keys | any YAML value | No | Preserve verbatim on read and write |

Preservation contract:

1. Unknown frontmatter fields must round-trip.
2. Unknown fields must not be dropped or reordered intentionally.
3. Importers must tolerate extra keys from other tools.

## Sidecar Schema

Sidecar stores non-Markdown fidelity only.
It must not duplicate plain textual body content.

```json
{
  "schema_version": 1,
  "entry_id": "8f14e45f-ceea-467e-9c9a-3f6b30930f39",
  "generated_at": "2024-01-15T21:14:03-05:00",
  "blocks": [],
  "attachments": [],
  "revisions": []
}
```

| Field | Type | Rule |
|---|---|---|
| `schema_version` | integer | Required for migrations |
| `entry_id` | UUID string | Must equal frontmatter `id` |
| `generated_at` | RFC 3339 datetime | Last sidecar generation timestamp |
| `blocks` | array | Rich block annotations/widgets |
| `attachments` | array | Metadata for binaries under `.penna/attachments` |
| `revisions` | array | Optional app-level revision metadata |

## Domain Model Expectations

Domain entities are storage-agnostic.
Adapters are responsible for Markdown/YAML/JSON mapping.

Expected domain concepts:

- `Entry`: id, metadata, body document, attachments, preserved unknown frontmatter.
- `Journal`: collection/root metadata and entry references.
- `Tag`: normalized tag value.
- `Attachment`: attachment identity and relative path.
- `MergeConflict`: explicit unresolved merge state.

## Import/Export Contracts

Importer and exporter behavior must be best-effort and non-destructive.

1. Import must not hard-fail on unsupported syntax.
2. Unsupported constructs should fall back to plain text or raw block representation.
3. Export must produce valid Markdown even when rich nodes cannot be fully represented.
4. Exporter strips sidecar pointer by default when producing portable output.
5. Frontmatter must be preserved, including unknown fields.

## Invariants

1. Every persisted entry has a stable `id`.
2. `updated_at` is greater than or equal to `created_at`.
3. Sidecar `entry_id` must match entry frontmatter `id`.
4. Removing sidecar cannot remove body text.
5. Filename changes must not change identity.

## Testing Requirements

At minimum, maintain tests for:

1. Markdown plus frontmatter import round-trip.
2. Unknown frontmatter preservation.
3. Sidecar mismatch detection (`entry_id` integrity).
4. Malformed Markdown graceful degradation.
5. Export behavior that omits sidecar metadata by default.

## Related Documents

- [Architecture](./ARCHITECTURE.md)
- [ADRs](./ADR/)
