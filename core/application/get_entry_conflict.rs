use crate::domain::EntryConflict;
use crate::ports::{ConflictView, RepositoryError};

pub struct GetEntryConflictUseCase<C: ConflictView> {
    conflicts: C,
}

impl<C: ConflictView> GetEntryConflictUseCase<C> {
    pub fn new(conflicts: C) -> Self {
        Self { conflicts }
    }

    pub fn execute(&self, id: &str) -> Result<Option<EntryConflict>, RepositoryError> {
        self.conflicts.entry_conflict(id)
    }
}
