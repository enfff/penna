use crate::domain::AttachmentMeta;
use crate::ports::{AttachmentStore, RepositoryError};

/// Sanity cap for a single attachment (ADR 0012).
pub const MAX_ATTACHMENT_BYTES: usize = 32 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AddAttachmentError {
    NotFound(String),
    InvalidName(String),
    TooLarge { size: usize, max: usize },
    Repository(RepositoryError),
}

pub struct AddAttachmentUseCase<S: AttachmentStore> {
    store: S,
}

impl<S: AttachmentStore> AddAttachmentUseCase<S> {
    pub fn new(store: S) -> Self {
        Self { store }
    }

    pub fn execute(
        &self,
        id: &str,
        name: &str,
        data: &[u8],
    ) -> Result<AttachmentMeta, AddAttachmentError> {
        validate_attachment_name(name).map_err(AddAttachmentError::InvalidName)?;

        if data.len() > MAX_ATTACHMENT_BYTES {
            return Err(AddAttachmentError::TooLarge {
                size: data.len(),
                max: MAX_ATTACHMENT_BYTES,
            });
        }

        self.store
            .add_attachment(id, name, data)
            .map_err(AddAttachmentError::Repository)
    }
}

/// Rejects empty names, path separators, and traversal attempts so the
/// stored file can never escape `<entry_id>/`.
pub fn validate_attachment_name(name: &str) -> Result<(), String> {
    if name.is_empty() || name.len() > 255 {
        return Err(format!("invalid attachment name length: {}", name.len()));
    }
    if name == "." || name == ".." || name.contains('/') || name.contains('\\') {
        return Err(format!("attachment name must be a plain file name: {name}"));
    }
    Ok(())
}
