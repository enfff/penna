use crate::domain::AttachmentMeta;
use crate::ports::{AttachmentStore, RepositoryError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RemoveAttachmentError {
    NotFound(String),
    Repository(RepositoryError),
}

pub struct RemoveAttachmentUseCase<S: AttachmentStore> {
    store: S,
}

impl<S: AttachmentStore> RemoveAttachmentUseCase<S> {
    pub fn new(store: S) -> Self {
        Self { store }
    }

    pub fn execute(&self, id: &str, name: &str) -> Result<Vec<AttachmentMeta>, RemoveAttachmentError> {
        self.store
            .remove_attachment(id, name)
            .map_err(|err| match err {
                RepositoryError::NotFound(what) => RemoveAttachmentError::NotFound(what),
                other => RemoveAttachmentError::Repository(other),
            })
    }
}

pub struct ListAttachmentsUseCase<S: AttachmentStore> {
    store: S,
}

impl<S: AttachmentStore> ListAttachmentsUseCase<S> {
    pub fn new(store: S) -> Self {
        Self { store }
    }

    pub fn execute(&self, id: &str) -> Result<Vec<AttachmentMeta>, RepositoryError> {
        self.store.list_attachments(id)
    }
}

pub struct GetAttachmentUseCase<S: AttachmentStore> {
    store: S,
}

impl<S: AttachmentStore> GetAttachmentUseCase<S> {
    pub fn new(store: S) -> Self {
        Self { store }
    }

    pub fn execute(&self, id: &str, name: &str) -> Result<Option<Vec<u8>>, RepositoryError> {
        self.store.get_attachment(id, name)
    }
}
