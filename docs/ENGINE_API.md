# ENGINE_API.md

Purpose: frontend integration contract for Penna engine v1.

## Scope

Engine API is frontend-agnostic. Any frontend (web, desktop, mobile, CLI) calls these methods through host bridge (IPC/RPC/direct Rust binding).

## Data Format Rules

1. Entry files are plain Markdown only.
2. Entry filename/id format is `YYYYMMDDHHmm.md`.
3. Engine resolves same-minute collisions by moving to next available minute slot.
4. Tags metadata storage: `.penna/YYYYMMDDHHmm.json`.
5. Sidecar JSON v1 shape: `{ "tags": ["..."] }`.
6. Global tag catalog storage: `.penna/tags.json`.

## Methods

## Tags + Sidecar Behavior (v1)

Frontend keeps using existing entry APIs.
No new frontend-facing tag API required.

Flow:

1. Send tags in `create_entry` / `update_entry` request.
2. Read tags from `get_entry` / `list_entries` response.
3. Engine/adapters persist tags in `.penna/<entry_id>.json`.
4. Missing sidecar is non-fatal; tags default to empty list.

## Repository Lifecycle APIs

These methods are implemented for remote-first onboarding and explicit sync controls.

### clone_journal

Purpose: clone remote git journal to local filesystem, then open engine session.

Input:

```json
{
  "remote_url": "https://example.com/user/journal.git",
  "local_parent_dir": "/home/user/Documents",
  "directory_name": "my-journal"
}
```

Output:

```json
{
  "session_id": "session-1754733553000000000",
  "repo_path": "/home/user/Documents/my-journal"
}
```

### resolve_journal_path

Purpose: return canonical absolute path for currently connected session.

Input:

```json
{ "session_id": "session-1754733553000000000" }
```

Output:

```json
{ "repo_path": "/home/user/Documents/my-journal" }
```

### pull_journal

Purpose: explicit pull-only operation for frontends that want manual pull button.

Input:

```json
{ "session_id": "session-1754733553000000000" }
```

Output:

```json
{
  "status": "up_to_date|pulled|no_remote|no_branch|diverged",
  "branch": "master",
  "ahead": null,
  "behind": null
}
```

### push_journal

Purpose: explicit push-only operation for frontends that want manual push button.

Input:

```json
{ "session_id": "session-1754733553000000000" }
```

Output:

```json
{
  "status": "up_to_date|pushed|no_remote|no_branch|diverged",
  "branch": "master",
  "ahead": null,
  "behind": null
}
```

### list_tags

Input:

```json
{ "session_id": "session-1754733553000000000" }
```

Output:

```json
{ "tags": ["daily", "work"] }
```

### add_tag

Input:

```json
{ "session_id": "session-1754733553000000000", "tag": "work" }
```

Output:

```json
{ "tags": ["daily", "work"] }
```

### remove_tag

Input:

```json
{ "session_id": "session-1754733553000000000", "tag": "work" }
```

Output:

```json
{ "tags": ["daily"] }
```

Behavior:

1. Removes selected tag from global catalog.
2. Removes selected tag from all notes.

### update_tag

Input:

```json
{
  "session_id": "session-1754733553000000000",
  "old_tag": "daily",
  "new_tag": "journal"
}
```

Output:

```json
{ "tags": ["journal", "work"] }
```

Behavior:

1. Renames selected tag in global catalog.
2. Renames selected tag across all notes.

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

## Attachments (ADR 0012)

Attachments live in a per-entry directory `journal-root/<id>/` next to
`<id>.md`. Entries reference them with relative links
(`![photo](202608241800/photo.png)`). The sidecar manifest
`.penna/<id>.json` gains an optional additive field:

```json
{ "tags": ["trip"], "attachments": [{ "name": "photo.png", "bytes": 81234 }] }
```

Methods (all require an open session):

### add_attachment

Input: `{ session_id, entry_id, name, data (bytes) }`.
Names must be plain file names (no separators, no traversal); payloads over
32 MiB are rejected with code `VALIDATION`. The entry must already exist.
Output: `{ "name": "photo.png", "bytes": 81234 }`.

### list_attachments

Input: `{ session_id, entry_id }`.
Output: `{ "attachments": [{ "name": "...", "bytes": 0 }] }`.

### get_attachment

Input: `{ session_id, entry_id, name }`.
Output: `{ "data": <bytes|null> }` — null when the attachment is absent.

### remove_attachment

Input: `{ session_id, entry_id, name }`.
Output: remaining `{ "attachments": [...] }`.

`delete_entry` removes the Markdown file, the `<id>/` directory, and
refreshes the sidecar in one commit.

## Conflict Resolution (ADR 0006)

When a sync report has status `diverged`, `conflicts` lists entry ids whose
bodies changed differently on both machines. The frontend merge-editor flow:

1. `get_entry_conflict` returns the three raw Markdown versions for the
   editor (base = common ancestor, ours = local, theirs = remote).

Input:

```json
{ "session_id": "session-...", "id": "202608241500" }
```

Output (null when the entry is not conflicted):

```json
{
  "conflict": {
    "entry_id": "202608241500",
    "base": "# Title\n\nancestor text",
    "ours": "# Title\n\nlocal text",
    "theirs": "# Title\n\nremote text"
  }
}
```

2. The user edits in-app; quick presets ("keep ours"/"keep theirs") are
   frontend shortcuts producing the same resolved text.
3. `resolve_entry_conflict` writes the user's body as a normal update
   commit; title, tags, and created_at stay from the local side.

Input:

```json
{ "session_id": "session-...", "id": "202608241500", "resolved_body": "# Title\n\nmerged text" }
```

4. After all entries are resolved, `reconcile_journal` creates the merge
   commit: conflicted bodies keep the resolved local text, clean changes
   from both sides auto-merge, and tag sidecars merge by set union.

Output:

```json
{ "status": "reconciled", "branch": "master", "ahead": null, "behind": null, "conflicts": [] }
```

5. Push as usual afterwards; the remote fast-forwards to the reconciled
   history.

Pushing while still diverged is refused (`diverged` status) so remote work
is never overwritten silently.

## Authentication (ADR 0010)

Sync methods resolve credentials automatically, in order:

1. SSH remotes: the user's ssh-agent.
2. HTTPS remotes: the provider-neutral `PENNA_GIT_TOKEN` env var, then the
   OS keychain (service `penna`, user = remote URL). The token is sent as
   the basic-auth password with a neutral username; works with any git
   server that accepts token auth (GitHub, GitLab, Gitea, Azure DevOps,
   self-hosted).
3. Local/file remotes need no credentials.

When authentication is required but unavailable, sync fails with code
`REPO` and a message naming the remote. Frontends prompt for the secret,
may persist it to the keychain via the engine's credential store, then
retry.

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
