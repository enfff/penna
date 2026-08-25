# Engine API Copy/Paste

Mirror of the current `penna-engine` public methods. The narrative
contract lives in [ENGINE_API.md](./ENGINE_API.md); this file exists so
frontend code can paste exact signatures. Update both in the same change.

## Rust Methods

```rust
pub fn connect_journal<P: AsRef<std::path::Path>>(&self, repo_path: P) -> Result<JournalSession, EngineError>;
pub fn clone_journal(&self, request: CloneJournalRequest) -> Result<JournalSession, EngineError>;
pub fn resolve_journal_path(&self, session_id: &str) -> Result<ResolveJournalPathResponse, EngineError>;
pub fn journal_status(&self, session_id: &str) -> Result<JournalStatus, EngineError>;
pub fn disconnect_journal(&self, session_id: &str) -> Result<(), EngineError>;

pub fn list_entries(&self, session_id: &str) -> Result<Vec<Entry>, EngineError>;
pub fn get_entry(&self, session_id: &str, id: &str) -> Result<Option<Entry>, EngineError>;
pub fn create_entry(&self, session_id: &str, request: CreateEntryRequest) -> Result<Entry, EngineError>;
pub fn create_entry_api(&self, request: CreateEntryApiRequest) -> Result<EntryDto, EngineError>;
pub fn update_entry(&self, session_id: &str, request: UpdateEntryRequest) -> Result<Entry, EngineError>;
pub fn delete_entry(&self, session_id: &str, id: &str) -> Result<(), EngineError>;

// Network I/O — call from a worker thread (ADR 0014 + ENGINE_API.md).
pub fn sync_journal(&self, session_id: &str) -> Result<SyncReport, EngineError>;
pub fn pull_journal(&self, session_id: &str) -> Result<SyncReport, EngineError>;
pub fn push_journal(&self, session_id: &str) -> Result<SyncReport, EngineError>;

// Conflict flow (ADR 0014): markers land in working files; these are helpers.
pub fn get_entry_conflict(&self, session_id: &str, id: &str) -> Result<Option<EntryConflict>, EngineError>;
pub fn resolve_entry_conflict(&self, session_id: &str, id: &str, resolved_body: &str) -> Result<Entry, EngineError>;
pub fn reconcile_journal(&self, session_id: &str) -> Result<SyncReport, EngineError>;

// Attachments (ADR 0012). Names must be plain file names; cap 32 MiB.
pub fn add_attachment(&self, session_id: &str, entry_id: &str, name: &str, data: Vec<u8>) -> Result<AttachmentMeta, EngineError>;
pub fn get_attachment(&self, session_id: &str, entry_id: &str, name: &str) -> Result<Option<Vec<u8>>, EngineError>;
pub fn list_attachments(&self, session_id: &str, entry_id: &str) -> Result<Vec<AttachmentMeta>, EngineError>;
pub fn remove_attachment(&self, session_id: &str, entry_id: &str, name: &str) -> Result<Vec<AttachmentMeta>, EngineError>;

pub fn list_tags(&self, session_id: &str) -> Result<Vec<String>, EngineError>;
pub fn add_tag(&self, session_id: &str, tag: &str) -> Result<Vec<String>, EngineError>;
pub fn remove_tag(&self, session_id: &str, tag: &str) -> Result<Vec<String>, EngineError>;
pub fn update_tag(&self, session_id: &str, old_tag: &str, new_tag: &str) -> Result<Vec<String>, EngineError>;
pub fn sidecar_integrity_status(&self, entry_id: &str, sidecar_json: Option<&str>) -> SidecarIntegrityReport;
```

## Error Shape

```rust
pub const PUBLIC_ERROR_CODES: [&str; 5] = [
    "NOT_CONNECTED", "IO", "REPO", "VALIDATION", "CONFLICT",
];

pub struct EngineErrorDto { pub code: String, pub message: String }
```

`code` is the contract; `message` is diagnostics only (ADR 0009).

## Sync Report

```rust
pub struct SyncReport {
    pub status: String,        // up_to_date | pulled | pushed | no_remote | no_branch | diverged | reconciled
    pub branch: Option<String>,
    pub ahead: Option<usize>,
    pub behind: Option<usize>,
    pub conflicts: Vec<String>, // conflicted entry ids when diverged
}
```

## Journal Status

```rust
pub struct JournalStatus {
    pub session_id: String,
    pub repo_path: String,
    pub branch: Option<String>,
    pub head_commit: Option<String>,
    pub is_dirty: bool,
    pub merge_in_progress: bool,
    pub conflicted_paths: Vec<String>,
    pub connected_at: String,
}
```

## Threading

All calls are synchronous. Anything named sync/pull/push/clone performs
network I/O and must run on a worker thread (see ENGINE_API.md
"Threading Contract"). Everything else is local disk only.

## Storage Layout

- Entry body: `<YYYYMMDDHHmm>.md` at journal root
- Entry sidecar: `.penna/<id>.json` → `{ "tags": [...], "attachments": [{"name","bytes"}] }`
- Global tag catalog: `.penna/tags.json`
- Attachments: `<id>/<file>` next to the entry file (ADR 0012)
- Conflict state: standard git markers in working files mid-merge (ADR 0014)
