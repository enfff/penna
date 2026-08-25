use std::path::{Path, PathBuf};

use penna_core::application::CloneJournalUseCase;
use penna_core::ports::{JournalClone, RepositoryError};

#[derive(Clone)]
struct FakeJournalClone {
    result: Result<(), RepositoryError>,
}

impl JournalClone for FakeJournalClone {
    fn clone_journal(&self, _remote_url: &str, _local_path: &Path) -> Result<(), RepositoryError> {
        self.result.clone()
    }
}

#[test]
fn clone_journal_calls_clone_port() {
    let use_case = CloneJournalUseCase::new(FakeJournalClone { result: Ok(()) });

    let result = use_case.execute(
        "https://example.com/repo.git",
        PathBuf::from("/tmp/journal"),
    );

    assert!(result.is_ok());
}

#[test]
fn clone_journal_propagates_repository_error() {
    let use_case = CloneJournalUseCase::new(FakeJournalClone {
        result: Err(RepositoryError::Storage("clone failed".to_string())),
    });

    let result = use_case.execute(
        "https://example.com/repo.git",
        PathBuf::from("/tmp/journal"),
    );

    assert_eq!(
        result,
        Err(RepositoryError::Storage("clone failed".to_string()))
    );
}
