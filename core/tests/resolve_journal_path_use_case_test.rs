use std::path::PathBuf;

use penna_core::application::ResolveJournalPathUseCase;
use penna_core::ports::{JournalPath, RepositoryError};

#[derive(Clone)]
struct FakeJournalPath {
    result: Result<PathBuf, RepositoryError>,
}

impl JournalPath for FakeJournalPath {
    fn resolve_path(&self) -> Result<PathBuf, RepositoryError> {
        self.result.clone()
    }
}

#[test]
fn resolve_journal_path_returns_canonical_path() {
    let use_case = ResolveJournalPathUseCase::new(FakeJournalPath {
        result: Ok(PathBuf::from("/home/user/journal")),
    });

    let path = use_case.execute().expect("path should resolve");

    assert_eq!(path, PathBuf::from("/home/user/journal"));
}

#[test]
fn resolve_journal_path_propagates_error() {
    let use_case = ResolveJournalPathUseCase::new(FakeJournalPath {
        result: Err(RepositoryError::Storage("path unavailable".to_string())),
    });

    let result = use_case.execute();

    assert_eq!(
        result,
        Err(RepositoryError::Storage("path unavailable".to_string()))
    );
}
