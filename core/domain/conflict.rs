use serde::{Deserialize, Serialize};

/// Three-way view of a diverged entry (ADR 0006). Values are raw
/// Markdown file contents so frontends can render any merge editor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntryConflict {
    pub entry_id: String,
    /// Common ancestor version.
    pub base: String,
    /// Local working version.
    pub ours: String,
    /// Remote version.
    pub theirs: String,
}
