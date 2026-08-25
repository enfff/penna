use crate::domain::{AttachmentMeta, Document, Entry, EntryConflict, Sidecar};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RepositoryError {
    Storage(String),
    NotFound(String),
    AuthRequired(String),
}

pub trait EntryRepository: Send + Sync {
    fn get(&self, id: &str) -> Result<Option<Entry>, RepositoryError>;
    fn save(&self, entry: &Entry) -> Result<(), RepositoryError>;
    fn delete(&self, id: &str) -> Result<(), RepositoryError>;
    fn list(&self) -> Result<Vec<Entry>, RepositoryError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyncResult {
    UpToDate { branch: String },
    NoRemote,
    NoBranch,
    Pulled { branch: String },
    Pushed { branch: String },
    Diverged { branch: String, ahead: usize, behind: usize },
}

pub trait JournalSync: Send + Sync {
    fn sync(&self) -> Result<SyncResult, RepositoryError>;
    fn pull(&self) -> Result<SyncResult, RepositoryError>;
    fn push(&self) -> Result<SyncResult, RepositoryError>;
}

/// Three-way conflict view for diverged journals (ADR 0006).
pub trait ConflictView: Send + Sync {
    fn list_conflicted_ids(&self) -> Result<Vec<String>, RepositoryError>;
    fn entry_conflict(&self, id: &str) -> Result<Option<EntryConflict>, RepositoryError>;
    /// Creates the reconciling merge commit. Local side wins conflicted
    /// paths; clean paths auto-merge.
    fn reconcile_with_remote(&self) -> Result<(), RepositoryError>;
}

/// Attachment storage under per-entry directories (ADR 0012). Files are
/// plain git blobs; the sidecar manifest tracks name and size.
pub trait AttachmentStore: Send + Sync {
    fn list_attachments(&self, id: &str) -> Result<Vec<AttachmentMeta>, RepositoryError>;
    fn get_attachment(&self, id: &str, name: &str) -> Result<Option<Vec<u8>>, RepositoryError>;
    fn add_attachment(
        &self,
        id: &str,
        name: &str,
        data: &[u8],
    ) -> Result<AttachmentMeta, RepositoryError>;
    fn remove_attachment(&self, id: &str, name: &str) -> Result<Vec<AttachmentMeta>, RepositoryError>;
}

pub trait JournalClone: Send + Sync {
    fn clone_journal(&self, remote_url: &str, local_path: &Path) -> Result<(), RepositoryError>;
}

pub trait JournalPath: Send + Sync {
    fn resolve_path(&self) -> Result<PathBuf, RepositoryError>;
}

pub trait TagCatalog: Send + Sync {
    fn list_tags(&self) -> Result<Vec<String>, RepositoryError>;
    fn add_tag(&self, tag: &str) -> Result<Vec<String>, RepositoryError>;
    fn remove_tag(&self, tag: &str) -> Result<Vec<String>, RepositoryError>;
    fn update_tag(&self, old_tag: &str, new_tag: &str) -> Result<Vec<String>, RepositoryError>;
}

use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileSystemError {
    Io(String),
    NotFound(String),
}

impl fmt::Display for FileSystemError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FileSystemError::Io(msg) => write!(f, "IO error: {}", msg),
            FileSystemError::NotFound(msg) => write!(f, "Not found: {}", msg),
        }
    }
}

impl std::error::Error for FileSystemError {}

pub trait FileSystem: Send + Sync {
    fn read(&self, path: &Path) -> Result<Vec<u8>, FileSystemError>;
    fn write(&self, path: &Path, data: &[u8]) -> Result<(), FileSystemError>;
    fn exists(&self, path: &Path) -> bool;
    fn create_dir_all(&self, path: &Path) -> Result<(), FileSystemError>;
}

pub trait MarkdownImporter: Send + Sync {
    fn import(&self, markdown: &str, frontmatter: &str) -> Result<Document, Box<dyn std::error::Error + Send + Sync>>;
}

pub trait MarkdownExporter: Send + Sync {
    fn export(&self, document: &Document) -> Result<String, Box<dyn std::error::Error + Send + Sync>>;
    
    fn export_with_sidecar(
        &self,
        document: &Document,
        sidecar: Option<&Sidecar>,
        include_sidecar: bool,
    ) -> Result<(String, Option<String>), Box<dyn std::error::Error + Send + Sync>>;
}
