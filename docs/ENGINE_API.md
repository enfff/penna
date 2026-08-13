# ENGINE_API.md

Purpose: frontend integration contract for Penna engine v1.

## Scope

Engine API is frontend-agnostic. Any frontend (web, desktop, mobile, CLI) calls these methods through host bridge (IPC/RPC/direct Rust binding).

## Data Format Rules

1. Entry files are plain Markdown only.
2. Entry filename/id format is `YYYYMMDDHHmm.md`.
3. Engine resolves same-minute collisions by moving to next available minute slot.
4. Planned tags metadata storage: `.penna/YYYYMMDDHHmm.json`.
5. Planned sidecar JSON v1 shape: `{ "tags": ["..."] }`.

## Methods

## Tags + Sidecar Behavior (Planned v1)

Frontend keeps using existing entry APIs.
No new frontend-facing tag API required.

Flow:

1. Send tags in `create_entry` / `update_entry` request.
2. Read tags from `get_entry` / `list_entries` response.
3. Engine/adapters persist tags in `.penna/<entry_id>.json`.
4. Missing sidecar is non-fatal; tags default to empty list.

### connect_journal

Input:

```json
{ "repo_path": "/absolute/or/user-chosen/path" }
```

Output:

```json
{
  "session_id": "session-1754733553000000000",
  "repo_path": "/absolute/or/user-chosen/path"
}
```

Behavior:

1. Creates directory if missing.
2. Opens existing git repo or initializes new one.
3. Registers active engine session.

### journal_status

Input:

```json
{ "session_id": "session-1754733553000000000" }
```

Output:

```json
{
  "session_id": "session-1754733553000000000",
  "repo_path": "/absolute/or/user-chosen/path",
  "branch": "master",
  "head_commit": "a1b2c3d4...",
  "is_dirty": false,
  "connected_at": "2026-08-09T11:17:42+00:00"
}
```

### disconnect_journal

Input:

```json
{ "session_id": "session-1754733553000000000" }
```

Output:

```json
{ "ok": true }
```

### list_entries

Input:

```json
{ "session_id": "session-1754733553000000000" }
```

Output:

```json
{
  "entries": [
    {
      "id": { "0": "202608091130" },
      "title": "Entry title",
      "body": "# Heading\n\nplain markdown",
      "tags": ["work"],
      "created_at": "2026-08-09T11:30:00+00:00",
      "updated_at": "2026-08-09T11:30:00+00:00"
    }
  ]
}
```

### get_entry

Input:

```json
{ "session_id": "session-1754733553000000000", "id": "202608091130" }
```

Output:

```json
{
  "entry": {
    "id": { "0": "202608091130" },
    "title": "Entry title",
    "body": "plain markdown",
    "tags": [],
    "created_at": "2026-08-09T11:30:00+00:00",
    "updated_at": "2026-08-09T11:30:00+00:00"
  }
}
```

### create_entry

Input:

```json
{
  "session_id": "session-1754733553000000000",
  "request": {
    "title": "New entry",
    "body": "plain markdown",
    "tags": ["daily"]
  }
}
```

Output:

```json
{
  "entry": {
    "id": { "0": "202608091131" },
    "title": "New entry",
    "body": "plain markdown",
    "tags": ["daily"],
    "created_at": "2026-08-09T11:31:12+00:00",
    "updated_at": "2026-08-09T11:31:12+00:00"
  }
}
```

### update_entry

Input:

```json
{
  "session_id": "session-1754733553000000000",
  "request": {
    "id": "202608091131",
    "title": "Updated title",
    "body": "updated plain markdown",
    "tags": ["daily", "edited"]
  }
}
```

Output:

```json
{
  "entry": {
    "id": { "0": "202608091131" },
    "title": "Updated title",
    "body": "updated plain markdown",
    "tags": ["daily", "edited"],
    "created_at": "2026-08-09T11:31:12+00:00",
    "updated_at": "2026-08-09T11:35:05+00:00"
  }
}
```

### delete_entry

Input:

```json
{ "session_id": "session-1754733553000000000", "id": "202608091131" }
```

Output:

```json
{ "ok": true }
```

### sync_journal

Input:

```json
{ "session_id": "session-1754733553000000000" }
```

Output (up to date):

```json
{
  "status": "up_to_date",
  "branch": "master",
  "ahead": null,
  "behind": null
}
```

Output (pushed):

```json
{
  "status": "pushed",
  "branch": "master",
  "ahead": null,
  "behind": null
}
```

Output (pulled):

```json
{
  "status": "pulled",
  "branch": "master",
  "ahead": null,
  "behind": null
}
```

Output (no remote):

```json
{
  "status": "no_remote",
  "branch": null,
  "ahead": null,
  "behind": null
}
```

Output (diverged):

```json
{
  "status": "diverged",
  "branch": "master",
  "ahead": 2,
  "behind": 1
}
```

### sidecar_integrity_status

Input:

```json
{
  "entry_id": "202608091131",
  "sidecar_json": "{\"tags\":[\"daily\",\"edited\"]}"
}
```

Output (ok):

```json
{
  "status": "ok",
  "expected_entry_id": null,
  "actual_entry_id": null,
  "reason": null
}
```

Output (mismatch):

```json
{
  "status": "mismatch",
  "expected_entry_id": "202608091131",
  "actual_entry_id": "wrong-id",
  "reason": null
}
```

Output (missing):

```json
{
  "status": "missing",
  "expected_entry_id": null,
  "actual_entry_id": null,
  "reason": null
}
```

Output (malformed):

```json
{
  "status": "malformed",
  "expected_entry_id": null,
  "actual_entry_id": null,
  "reason": "expected value at line 1 column 1"
}
```

## Error Contract

Engine returns typed errors internally. Bridge should map to consistent frontend shape:

```json
{
  "code": "NOT_CONNECTED|IO|REPO|VALIDATION|CONFLICT",
  "message": "human readable detail"
}
```

## Integration Notes

1. Keep `session_id` in frontend state after successful connect.
2. Do not call adapters directly from frontend.
3. Run all entry operations via engine session methods.
