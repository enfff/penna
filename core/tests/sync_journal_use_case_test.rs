use penna_core::application::SyncJournalUseCase;
use penna_core::ports::{JournalSync, RepositoryError, SyncResult};

#[derive(Clone)]
struct FakeJournalSync {
    result: Result<SyncResult, RepositoryError>,
}

impl JournalSync for FakeJournalSync {
    fn sync(&self) -> Result<SyncResult, RepositoryError> {
        self.result.clone()
    }

    fn pull(&self) -> Result<SyncResult, RepositoryError> {
        self.result.clone()
    }

    fn push(&self) -> Result<SyncResult, RepositoryError> {
        self.result.clone()
    }
}

#[test]
fn sync_journal_returns_port_result() {
    let use_case = SyncJournalUseCase::new(FakeJournalSync {
        result: Ok(SyncResult::Pushed {
            branch: "master".to_string(),
        }),
    });

    let result = use_case.execute().expect("sync should succeed");

    assert_eq!(
        result,
        SyncResult::Pushed {
            branch: "master".to_string(),
        }
    );
}

#[test]
fn sync_journal_propagates_repository_error() {
    let use_case = SyncJournalUseCase::new(FakeJournalSync {
        result: Err(RepositoryError::Storage("push failed".to_string())),
    });

    let result = use_case.execute();

    assert_eq!(
        result,
        Err(RepositoryError::Storage("push failed".to_string()))
    );
}
