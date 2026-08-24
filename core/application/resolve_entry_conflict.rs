use crate::domain::Entry;
use crate::ports::{EntryRepository, RepositoryError};

pub struct ResolveEntryConflictUseCase<R: EntryRepository> {
    repository: R,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolveEntryConflictError {
    NotFound(String),
    Repository(RepositoryError),
}

impl<R: EntryRepository> ResolveEntryConflictUseCase<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    /// Writes the user's merged body as a normal update commit (ADR 0006).
    /// Title, tags, and created_at stay from the local side.
    pub fn execute(&self, id: &str, resolved_body: &str) -> Result<Entry, ResolveEntryConflictError> {
        let mut entry = self
            .repository
            .get(id)
            .map_err(ResolveEntryConflictError::Repository)?
            .ok_or_else(|| ResolveEntryConflictError::NotFound(id.to_string()))?;

        entry.body = resolved_body.to_string();
        self.repository
            .save(&entry)
            .map_err(ResolveEntryConflictError::Repository)?;
        Ok(entry)
    }
}
