use penna_core::application::{
    GetEntryConflictUseCase, ResolveEntryConflictError, ResolveEntryConflictUseCase,
};
use penna_core::domain::{Entry, EntryConflict, EntryId};
use penna_core::ports::{ConflictView, EntryRepository, RepositoryError};
use std::sync::Mutex;

struct FakeConflictView {
    conflicts: Vec<EntryConflict>,
}

impl ConflictView for FakeConflictView {
    fn list_conflicted_ids(&self) -> Result<Vec<String>, RepositoryError> {
        Ok(self.conflicts.iter().map(|c| c.entry_id.clone()).collect())
    }

    fn entry_conflict(&self, id: &str) -> Result<Option<EntryConflict>, RepositoryError> {
        Ok(self
            .conflicts
            .iter()
            .find(|c| c.entry_id == id)
            .cloned())
    }

    fn reconcile_with_remote(&self) -> Result<(), RepositoryError> {
        Ok(())
    }
}

struct FakeRepository {
    entries: Mutex<Vec<Entry>>,
}

impl EntryRepository for FakeRepository {
    fn get(&self, id: &str) -> Result<Option<Entry>, RepositoryError> {
        Ok(self.entries.lock().unwrap()
            .iter()
            .find(|e| e.id.0 == id)
            .cloned())
    }

    fn save(&self, entry: &Entry) -> Result<(), RepositoryError> {
        let mut entries = self.entries.lock().unwrap();
        match entries.iter_mut().find(|e| e.id.0 == entry.id.0) {
            Some(existing) => *existing = entry.clone(),
            None => entries.push(entry.clone()),
        }
        Ok(())
    }

    fn delete(&self, id: &str) -> Result<(), RepositoryError> {
        self.entries.lock().unwrap().retain(|e| e.id.0 != id);
        Ok(())
    }

    fn list(&self) -> Result<Vec<Entry>, RepositoryError> {
        Ok(self.entries.lock().unwrap().clone())
    }
}

fn conflict_for(id: &str) -> EntryConflict {
    EntryConflict {
        entry_id: id.to_string(),
        base: "base text".to_string(),
        ours: "our text".to_string(),
        theirs: "their text".to_string(),
    }
}

#[test]
fn returns_conflict_when_entry_is_diverged() {
    let view = FakeConflictView {
        conflicts: vec![conflict_for("202608241200")],
    };
    let use_case = GetEntryConflictUseCase::new(view);

    let conflict = use_case
        .execute("202608241200")
        .expect("lookup should succeed")
        .expect("entry must be conflicted");

    assert_eq!(conflict.entry_id, "202608241200");
    assert_eq!(conflict.ours, "our text");
    assert_eq!(conflict.theirs, "their text");
    assert_eq!(conflict.base, "base text");
}

#[test]
fn returns_none_for_unconflicted_entry() {
    let view = FakeConflictView { conflicts: vec![] };
    let use_case = GetEntryConflictUseCase::new(view);

    let conflict = use_case
        .execute("202608241201")
        .expect("lookup should succeed");

    assert!(conflict.is_none());
}

#[test]
fn resolution_keeps_local_title_tags_created_at_and_writes_body() {
    let repo = FakeRepository {
        entries: Mutex::new(vec![Entry {
            id: EntryId("202608241200".to_string()),
            title: "Local title".to_string(),
            body: "our text".to_string(),
            tags: vec!["work".to_string()],
            created_at: "2026-08-01T10:00:00+00:00".to_string(),
            updated_at: "2026-08-20T10:00:00+00:00".to_string(),
        }]),
    };
    let use_case = ResolveEntryConflictUseCase::new(repo);

    let resolved = use_case
        .execute("202608241200", "manually merged prose")
        .expect("resolution should succeed");

    assert_eq!(resolved.title, "Local title");
    assert_eq!(resolved.body, "manually merged prose");
    assert_eq!(resolved.tags, vec!["work".to_string()]);
    assert_eq!(resolved.created_at, "2026-08-01T10:00:00+00:00");
}

#[test]
fn resolution_of_unknown_entry_is_not_found() {
    let repo = FakeRepository {
        entries: Mutex::new(vec![]),
    };
    let use_case = ResolveEntryConflictUseCase::new(repo);

    let err = use_case
        .execute("ghost", "text")
        .expect_err("unknown entry must fail");

    assert_eq!(err, ResolveEntryConflictError::NotFound("ghost".to_string()));
}
