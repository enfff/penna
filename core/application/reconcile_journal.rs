use crate::ports::{ConflictView, RepositoryError};

pub struct ReconcileJournalUseCase<C: ConflictView> {
    conflicts: C,
}

impl<C: ConflictView> ReconcileJournalUseCase<C> {
    pub fn new(conflicts: C) -> Self {
        Self { conflicts }
    }

    /// Creates the reconciling merge commit after all entry conflicts are
    /// resolved (ADR 0006). Local side wins conflicted bodies; tags merge
    /// by union; clean paths auto-merge.
    pub fn execute(&self) -> Result<(), RepositoryError> {
        self.conflicts.reconcile_with_remote()
    }
}
