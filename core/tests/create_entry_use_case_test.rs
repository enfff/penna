use std::sync::Mutex;

use penna_core::application::{CreateEntryInput, CreateEntryUseCase};
use penna_core::domain::Entry;
use penna_core::ports::{EntryRepository, RepositoryError};

#[derive(Default)]
struct FakeEntryRepository {
    entries: Mutex<Vec<Entry>>,
}

impl EntryRepository for FakeEntryRepository {
    fn get(&self, id: &str) -> Result<Option<Entry>, RepositoryError> {
        let entries = self.entries.lock().map_err(|_| RepositoryError::Storage("lock poisoned".to_string()))?;
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
        let entries = self.entries.lock().map_err(|_| RepositoryError::Storage("lock poisoned".to_string()))?;
        Ok(entries.clone())
    }
}

#[test]
fn create_entry_persists_entry() {
    let repository = FakeEntryRepository::default();
    let use_case = CreateEntryUseCase::new(repository);

    let created = use_case
        .execute(CreateEntryInput {
            id: "entry-1".to_string(),
            title: "First Entry".to_string(),
            body: "Today was productive.".to_string(),
            tags: vec!["work".to_string()],
            created_at: "2026-08-04T00:00:00Z".to_string(),
            updated_at: "2026-08-04T00:00:00Z".to_string(),
        })
        .expect("entry should be created");

    assert_eq!(created.title, "First Entry");
}

#[test]
fn create_entry_allows_blank_title() {
    // Blank titles must be accepted: "new blank note" and "delete heading
    // then save" are both valid user flows. The title is stored as given.
    let repository = FakeEntryRepository::default();
    let use_case = CreateEntryUseCase::new(repository);

    let created = use_case
        .execute(CreateEntryInput {
            id: "entry-2".to_string(),
            title: "   ".to_string(),
            body: "No title".to_string(),
            tags: Vec::new(),
            created_at: "2026-08-04T00:00:00Z".to_string(),
            updated_at: "2026-08-04T00:00:00Z".to_string(),
        })
        .expect("blank title must be accepted");

    // Stored verbatim: a blank title round-trips as a blank Markdown heading.
    assert_eq!(created.title, "   ");
}
