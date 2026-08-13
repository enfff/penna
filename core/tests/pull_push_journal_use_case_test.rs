use penna_core::application::{PullJournalUseCase, PushJournalUseCase};
use penna_core::ports::{JournalSync, RepositoryError, SyncResult};

#[derive(Clone)]
struct FakeJournalSync {
    pull_result: Result<SyncResult, RepositoryError>,
    push_result: Result<SyncResult, RepositoryError>,
    sync_result: Result<SyncResult, RepositoryError>,
}

impl JournalSync for FakeJournalSync {
    fn sync(&self) -> Result<SyncResult, RepositoryError> {
        self.sync_result.clone()
    }

    fn pull(&self) -> Result<SyncResult, RepositoryError> {
        self.pull_result.clone()
    }

    fn push(&self) -> Result<SyncResult, RepositoryError> {
        self.push_result.clone()
    }
}

#[test]
fn pull_journal_returns_pull_result() {
    let use_case = PullJournalUseCase::new(FakeJournalSync {
        pull_result: Ok(SyncResult::Pulled {
            branch: "master".to_string(),
        }),
        push_result: Ok(SyncResult::UpToDate {
            branch: "master".to_string(),
        }),
        sync_result: Ok(SyncResult::UpToDate {
            branch: "master".to_string(),
        }),
    });

    let result = use_case.execute().expect("pull should succeed");

    assert_eq!(
        result,
        SyncResult::Pulled {
            branch: "master".to_string(),
        }
    );
}

#[test]
fn push_journal_returns_push_result() {
    let use_case = PushJournalUseCase::new(FakeJournalSync {
        pull_result: Ok(SyncResult::UpToDate {
            branch: "master".to_string(),
        }),
        push_result: Ok(SyncResult::Pushed {
            branch: "master".to_string(),
        }),
        sync_result: Ok(SyncResult::UpToDate {
            branch: "master".to_string(),
        }),
    });

    let result = use_case.execute().expect("push should succeed");

    assert_eq!(
        result,
        SyncResult::Pushed {
            branch: "master".to_string(),
        }
    );
}
