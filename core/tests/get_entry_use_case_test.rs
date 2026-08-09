use penna_core::application::GetEntryUseCase;
use penna_core::domain::{Entry, EntryId};
use penna_core::ports::{EntryRepository, RepositoryError};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

struct FakeEntryRepository {
    entries: Arc<RwLock<HashMap<String, Entry>>>,
}

impl FakeEntryRepository {
    fn new() -> Self {
        Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    fn add_entry(&self, entry: Entry) {
        self.entries.write().unwrap().insert(entry.id.0.clone(), entry);
    }
}

impl EntryRepository for FakeEntryRepository {
    fn get(&self, id: &str) -> Result<Option<Entry>, RepositoryError> {
        Ok(self.entries.read().unwrap().get(id).cloned())
    }
    
    fn save(&self, entry: &Entry) -> Result<(), RepositoryError> {
        self.entries.write().unwrap().insert(entry.id.0.clone(), entry.clone());
        Ok(())
    }
    
    fn delete(&self, id: &str) -> Result<(), RepositoryError> {
        self.entries.write().unwrap().remove(id);
        Ok(())
    }
    
    fn list(&self) -> Result<Vec<Entry>, RepositoryError> {
        Ok(self.entries.read().unwrap().values().cloned().collect())
    }
}

#[test]
fn get_entry_returns_entry_when_exists() {
    let repo = FakeEntryRepository::new();
    
    let entry = Entry::new(
        EntryId("test-123".to_string()),
        "Test Entry".to_string(),
        "Test content".to_string(),
        vec![],
        "1234567890".to_string(),
        "1234567890".to_string(),
    ).unwrap();
    
    repo.add_entry(entry);
    
    let use_case = GetEntryUseCase::new(repo);
    let result = use_case.execute("test-123").unwrap();
    
    assert!(result.is_some());
    assert_eq!(result.unwrap().title, "Test Entry");
}

#[test]
fn get_entry_returns_none_when_not_found() {
    let repo = FakeEntryRepository::new();
    let use_case = GetEntryUseCase::new(repo);
    let result = use_case.execute("non-existent").unwrap();
    
    assert!(result.is_none());
}
