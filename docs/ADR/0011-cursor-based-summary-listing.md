# ADR 0011: Cursor-Based Summary Listing

- Status: Rejected
- Date: 2026-08-24

> **Rejected during review.** No replacement decision was made. The existing
> full-materialization `list_entries` contract remains authoritative. If
> listing scale ever becomes a real problem, draft a fresh ADR (offset
> pages, cursors, or lazy loading) — do not resurrect this one silently.

## Context

`list_entries` materializes every entry with its full body into a
`Vec<Entry>` (engine/src/lib.rs:373). A journal grows without bound; ten
years of daily entries means tens of thousands of files and megabytes of
body text loaded to render a list. Frontends need pages; the current API
cannot serve them.

Offset pagination is tempting but subtly wrong here: ADR 0004 allows new
entries to land at an *earlier* lexicographic slot (minute-shift on
collision), so a page computed by offset shifts whenever anything is
inserted before it.

## Decision

1. `Entry.id` values are `YYYYMMDDHHmm` strings, so lexicographic order is
   chronological order. Pagination cursors are entry ids.
2. `list_entries` evolves to accept optional parameters:
   - `limit`: maximum results,
   - `after_id` / `before_id`: exclusive cursor bounds.
   Absent parameters keep today's behavior (full list) during migration;
   the full-list form is deprecated for frontends and removed before 1.0.
3. Default ordering is id descending: newest first, the universal journal
   view. Callers may request ascending.
4. List results are summaries (`id`, `title`, `tags`, timestamps) without
   bodies. Bodies load through `get_entry`. The summary type lives in
   core/application; the domain `Entry` is unchanged.
5. Response includes `next_cursor` (or null) so frontends never compute ids
   themselves.
6. Search/filtering stays out of this decision; a scan-based search use case
   can reuse cursors later.

## Alternatives Considered

- **Offset/limit pages** (`page=3&size=50`). Breaks under backdated inserts,
  which this data model produces routinely.
- **Keep load-everything, optimize later.** Works until roughly the size of
  journal the product exists for; retrofitting the API shape after frontends
  exist costs more than doing it now.
- **SQLite full-text index as listing source.** Fast, but introduces a
  second truth that must be rebuilt from files; violates the
  filesystem-as-registry simplicity the project runs on.

## Consequences

### Positive

- Constant memory per page regardless of journal age.
- Stable pages under concurrent minute-shift inserts.
- Summaries let TUI/GUI render instantly and stream the rest.

### Negative

- Displaying an entry costs two calls (summary already in hand, body via
  `get_entry`) — acceptable for local in-process calls (ADR 0005).
- Two result shapes (summary vs full) must be kept coherent in the API doc.
