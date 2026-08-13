# Engine API Copy/Paste

Use this as single source for frontend integration.

## Rust Methods

```rust
pub fn connect_journal<P: AsRef<std::path::Path>>(&self, repo_path: P) -> Result<JournalSession, EngineError>;
pub fn journal_status(&self, session_id: &str) -> Result<JournalStatus, EngineError>;
pub fn disconnect_journal(&self, session_id: &str) -> Result<(), EngineError>;
pub fn list_entries(&self, session_id: &str) -> Result<Vec<Entry>, EngineError>;
pub fn get_entry(&self, session_id: &str, id: &str) -> Result<Option<Entry>, EngineError>;
pub fn create_entry(&self, session_id: &str, request: CreateEntryRequest) -> Result<Entry, EngineError>;
pub fn create_entry_api(&self, request: CreateEntryApiRequest) -> Result<EntryDto, EngineError>;
pub fn update_entry(&self, session_id: &str, request: UpdateEntryRequest) -> Result<Entry, EngineError>;
pub fn delete_entry(&self, session_id: &str, id: &str) -> Result<(), EngineError>;
pub fn sync_journal(&self, session_id: &str) -> Result<SyncReport, EngineError>;
pub fn list_tags(&self, session_id: &str) -> Result<Vec<String>, EngineError>;
pub fn add_tag(&self, session_id: &str, tag: &str) -> Result<Vec<String>, EngineError>;
pub fn remove_tag(&self, session_id: &str, tag: &str) -> Result<Vec<String>, EngineError>;
pub fn update_tag(&self, session_id: &str, old_tag: &str, new_tag: &str) -> Result<Vec<String>, EngineError>;
pub fn sidecar_integrity_status(&self, entry_id: &str, sidecar_json: Option<&str>) -> SidecarIntegrityReport;

// Proposed
pub fn clone_journal(&self, request: CloneJournalRequest) -> Result<JournalSession, EngineError>;
pub fn resolve_journal_path(&self, session_id: &str) -> Result<ResolveJournalPathResponse, EngineError>;
pub fn pull_journal(&self, session_id: &str) -> Result<SyncReport, EngineError>;
pub fn push_journal(&self, session_id: &str) -> Result<SyncReport, EngineError>;
```

## Planned Tags Sidecar Contract (v1)

No new engine methods required for frontend tag CRUD.
Frontend keeps using existing entry APIs with `tags` field.

Storage target (adapter-level):

- Entry body: `YYYYMMDDHHmm.md`
- Entry tags sidecar: `.penna/YYYYMMDDHHmm.json`

Sidecar JSON target shape:

```json
{
  "tags": ["work", "idea"]
}
```

Behavior target:

- `create_entry`/`update_entry`: persist `tags` into `.penna/<id>.json`.
- `get_entry`/`list_entries`: hydrate `tags` from `.penna/<id>.json`.
- missing sidecar: return empty tags, no hard failure.
- malformed sidecar: return empty tags + integrity warning path via `sidecar_integrity_status`.

Global tag catalog storage (implemented):

- Path: `.penna/tags.json`
- Shape:

```json
{
  "tags": ["work", "idea"]
}
```

## DTOs

```rust
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct JournalSession {
    pub session_id: String,
    pub repo_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct JournalStatus {
    pub session_id: String,
    pub repo_path: String,
    pub branch: Option<String>,
    pub head_commit: Option<String>,
    pub is_dirty: bool,
    pub connected_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CreateEntryRequest {
    pub title: String,
    pub body: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct UpdateEntryRequest {
    pub id: String,
    pub title: String,
    pub body: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CreateEntryApiRequest {
  pub session_id: String,
  pub title: String,
  pub body: String,
  pub tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct EntryDto {
  pub id: String,
  pub title: String,
  pub body: String,
  pub tags: Vec<String>,
  pub created_at: String,
  pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct EngineErrorDto {
  pub code: String,
  pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SidecarIntegrityReport {
  pub status: String,
  pub expected_entry_id: Option<String>,
  pub actual_entry_id: Option<String>,
  pub reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SyncReport {
  pub status: String,
  pub branch: Option<String>,
  pub ahead: Option<usize>,
  pub behind: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CloneJournalRequest {
  pub remote_url: String,
  pub local_parent_dir: String,
  pub directory_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ResolveJournalPathResponse {
  pub repo_path: String,
}
```

## JSON Payloads

```json
{
  "connect_journal": { "repo_path": "/path/to/journal" },
  "journal_status": { "session_id": "session-1754733553000000000" },
  "disconnect_journal": { "session_id": "session-1754733553000000000" },
  "list_entries": { "session_id": "session-1754733553000000000" },
  "get_entry": {
    "session_id": "session-1754733553000000000",
    "id": "202608091130"
  },
  "create_entry": {
    "session_id": "session-1754733553000000000",
    "request": {
      "title": "New entry",
      "body": "plain markdown",
      "tags": ["daily"]
    }
  },
  "create_entry_api": {
    "session_id": "session-1754733553000000000",
    "title": "New entry",
    "body": "plain markdown",
    "tags": ["daily"]
  },
  "update_entry": {
    "session_id": "session-1754733553000000000",
    "request": {
      "id": "202608091130",
      "title": "Updated",
      "body": "updated markdown",
      "tags": ["daily", "edited"]
    }
  },
  "delete_entry": {
    "session_id": "session-1754733553000000000",
    "id": "202608091130"
  },
  "list_tags": {
    "session_id": "session-1754733553000000000"
  },
  "add_tag": {
    "session_id": "session-1754733553000000000",
    "tag": "work"
  },
  "remove_tag": {
    "session_id": "session-1754733553000000000",
    "tag": "work"
  },
  "update_tag": {
    "session_id": "session-1754733553000000000",
    "old_tag": "work",
    "new_tag": "deep-work"
  },
  "sync_journal": {
    "session_id": "session-1754733553000000000"
  },
  "clone_journal": {
    "remote_url": "https://example.com/user/journal.git",
    "local_parent_dir": "/home/user/Documents",
    "directory_name": "my-journal"
  },
  "resolve_journal_path": {
    "session_id": "session-1754733553000000000"
  },
  "pull_journal": {
    "session_id": "session-1754733553000000000"
  },
  "push_journal": {
    "session_id": "session-1754733553000000000"
  },
  "sidecar_integrity_status": {
    "entry_id": "202608091130",
    "sidecar_json": "{\"tags\":[\"daily\",\"work\"]}"
  }
}
```

## Quick Real Repo Test

```bash
cargo run -p penna-engine --example create_entry_api -- /home/enf/Projects/penna-myjournal "Test title" "Test body"
```

## File Naming Rule

- Entry id and filename format: YYYYMMDDHHmm.md
- Content format: plain markdown
