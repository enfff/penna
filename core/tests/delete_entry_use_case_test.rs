use std::sync::{Arc, Mutex};

use penna_core::application::{DeleteEntryUseCase, GetEntryUseCase};
use penna_core::domain::{Entry, EntryId};
use penna_core::ports::{EntryRepository, RepositoryError};

#[derive(Clone, Default)]
struct FakeEntryRepository {
    entries: Arc<Mutex<Vec<Entry>>>,
}

impl EntryRepository for FakeEntryRepository {
    fn get(&self, id: &str) -> Result<Option<Entry>, RepositoryError> {
        let entries = self
            .entries
            .lock()
            .map_err(|_| RepositoryError::Storage("lock poisoned".to_string()))?;
        Ok(entries.iter().find(|e| e.id.0 == id).cloned())
    }

    fn save(&self, entry: &Entry) -> Result<(), RepositoryError> {
        let mut entries = self
            .entries
            .lock()
            .map_err(|_| RepositoryError::Storage("lock poisoned".to_string()))?;

        entries.push(entry.clone());
        Ok(())
    }

    fn delete(&self, id: &str) -> Result<(), RepositoryError> {
        let mut entries = self
            .entries
            .lock()
            .map_err(|_| RepositoryError::Storage("lock poisoned".to_string()))?;
        entries.retain(|e| e.id.0 != id);
        Ok(())
    }

    fn list(&self) -> Result<Vec<Entry>, RepositoryError> {
        let entries = self
            .entries
            .lock()
            .map_err(|_| RepositoryError::Storage("lock poisoned".to_string()))?;
        Ok(entries.clone())
    }
}

#[test]
fn delete_entry_removes_entry() {
    let repository = FakeEntryRepository::default();

    repository
        .save(&Entry::new(
            EntryId("202608091230".to_string()),
            "To Remove".to_string(),
            "Body".to_string(),
            vec![],
            "2026-08-09T12:30:00Z".to_string(),
            "2026-08-09T12:30:00Z".to_string(),
        )
        .expect("valid entry"))
        .expect("save should succeed");

    let delete_use_case = DeleteEntryUseCase::new(repository.clone());
    delete_use_case
        .execute("202608091230")
        .expect("delete should succeed");

    let get_use_case = GetEntryUseCase::new(repository);
    let deleted = get_use_case
        .execute("202608091230")
        .expect("get should succeed");

    assert!(deleted.is_none());
}