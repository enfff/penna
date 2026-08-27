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
  "merge_in_progress": false,
  "conflicted_paths": [],
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
      "id": "202608091130",
      "title": "Entry title",
      "body": "plain markdown",
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
    "id": "202608091130",
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
  "behind": null,
  "conflicts": []
}
```

Output (pushed):

```json
{
  "status": "pushed",
  "branch": "master",
  "ahead": null,
  "behind": null,
  "conflicts": []
}
```

Output (pulled):

```json
{
  "status": "pulled",
  "branch": "master",
  "ahead": null,
  "behind": null,
  "conflicts": []
}
```

Output (no remote):

```json
{
  "status": "no_remote",
  "branch": null,
  "ahead": null,
  "behind": null,
  "conflicts": []
}
```

Output (diverged):

```json
{
  "status": "diverged",
  "branch": "master",
  "ahead": 2,
  "behind": 1,
  "conflicts": ["202608250800"]
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

## Conflict Resolution (ADR 0014)

Divergence reconciles via a standard git **merge** (never rebase).
Conflicted entries carry plain `<<<<<<< / ======= / >>>>>>>` markers in
working files (`merge.conflictStyle = merge` is pinned per repository).

Flow:

1. `sync_journal` / `pull_journal` start the merge automatically when
   diverged, then report `{ "status": "diverged", ..., "conflicts": [ids] }`.
   Marker-less conflicts are resolved by policy at merge start: tag
   sidecars union-merge silently, modified-side-wins resurrects deleted
   files, both-deleted drops them.
2. Reads (`get_entry`, `list_entries`) always return working-tree content,
   so markers surface immediately.
3. The frontend renders its merge editor from the marked text (or asks
   `get_entry_conflict` for structured base/ours/theirs served from index
   stages).
4. Resolution writes clean text through the normal entry update path;
   while the merge is open, saves stage without committing.
5. After the last conflict is resolved, the next `sync_journal`
   auto-concludes: it commits with parents `[HEAD, MERGE_HEAD]`, clears
   `MERGE_HEAD`, and returns `pulled`. Pushing afterwards fast-forwards
   the remote.

`journal_status` grows two fields so detection never scans prose:

```json
{
  "merge_in_progress": true,
  "conflicted_paths": ["202608250800.md"]
}
```

Pushing while still diverged or mid-merge reports `diverged`; remote work
is never overwritten silently.

## Authentication (ADR 0010)

Network methods (`sync_journal`, `pull_journal`, `push_journal`,
`clone_journal`) resolve credentials automatically, in order:

1. SSH remotes: the user's ssh-agent.
2. HTTPS remotes: the provider-neutral `PENNA_GIT_TOKEN` env var, then the
   OS keychain (service `penna`, user = remote URL). The token is sent as
   the basic-auth password with a neutral username; works with any git
   server that accepts token auth (GitHub, GitLab, Gitea, Azure DevOps,
   self-hosted).
3. Local/file remotes need no credentials.

When authentication is required but unavailable, sync fails with code
`AUTH_REQUIRED` and `auth_remote` set to the remote URL (ADR 0015).
Frontends prompt for the secret, persist it via the credential store,
then retry.

### Credential Store (ADR 0015)

Per-remote, not session-scoped — a credential belongs to the remote, not
the open journal:

- `store_credential(remote_url, secret)` -> persist a token (e.g. an HTTPS
  PAT) in the platform secret store. Blank secrets are rejected with
  `VALIDATION`. The resolution path picks the token up on the next
  fetch/push/clone.
- `delete_credential(remote_url)` -> remove the stored credential
  (account rotation). Backend behavior for missing entries varies; check
  `has_credential` first when idempotency matters.
- `has_credential(remote_url)` -> true if a credential is stored.

Flow: sync returns `AUTH_REQUIRED` + `auth_remote` → frontend prompts →
`store_credential` → retry sync. No prompt or callback ever runs inside
the engine.

## Threading Contract

All methods are synchronous. Frontends must move calls onto worker
threads; nothing here is async. What each call can block on:

| Method | Blocks on |
|--------|-----------|
| `connect_journal`, `clone_journal` | filesystem; `clone_journal` also network + OS keychain lookup |
| `journal_status`, `resolve_journal_path` | local disk only |
| `list_entries`, `get_entry` | local disk only (timestamp cache: first read after new commits walks them once) |
| `create_entry`, `update_entry`, `delete_entry` | local disk only |
| `add_attachment`, `get_attachment`, `list_attachments`, `remove_attachment` | local disk only |
| `sync_journal`, `pull_journal`, `push_journal` | **network fetch/push** + possibly OS keychain lookup |
| `store_credential`, `delete_credential`, `has_credential` | OS keychain IPC only |
| tag and sidecar-integrity methods | local disk only |

Rule of thumb: anything with "sync", "pull", "push", or "clone" in the
name does network I/O. Everything else is local.

## Error Contract

Engine returns typed errors internally. Bridge should map to consistent frontend shape:

```json
{
  "code": "NOT_CONNECTED|IO|REPO|VALIDATION|CONFLICT|AUTH_REQUIRED",
  "message": "human readable detail",
  "auth_remote": "https://github.com/user/journal.git"
}
```

`auth_remote` is present if and only if `code` is `AUTH_REQUIRED`
(ADR 0015); it is omitted from the serialized shape otherwise.

## Integration Notes

1. Keep `session_id` in frontend state after successful connect.
2. Do not call adapters directly from frontend.
3. Run all entry operations via engine session methods.
