use std::sync::Mutex;

use penna_core::application::{
    AddTagError, AddTagUseCase, ListTagsUseCase, RemoveTagError, RemoveTagUseCase,
    UpdateTagError, UpdateTagUseCase,
};
use penna_core::ports::{RepositoryError, TagCatalog};

#[derive(Default)]
struct FakeTagCatalog {
    tags: Mutex<Vec<String>>,
}

impl TagCatalog for FakeTagCatalog {
    fn list_tags(&self) -> Result<Vec<String>, RepositoryError> {
        Ok(self.tags.lock().unwrap().clone())
    }

    fn add_tag(&self, tag: &str) -> Result<Vec<String>, RepositoryError> {
        let mut tags = self.tags.lock().unwrap();
        if !tags.iter().any(|t| t == tag) {
            tags.push(tag.to_string());
        }
        tags.sort();
        Ok(tags.clone())
    }

    fn remove_tag(&self, tag: &str) -> Result<Vec<String>, RepositoryError> {
        let mut tags = self.tags.lock().unwrap();
        tags.retain(|t| t != tag);
        Ok(tags.clone())
    }

    fn update_tag(&self, old_tag: &str, new_tag: &str) -> Result<Vec<String>, RepositoryError> {
        let mut tags = self.tags.lock().unwrap();
        if let Some(pos) = tags.iter().position(|t| t == old_tag) {
            tags[pos] = new_tag.to_string();
            tags.sort();
            tags.dedup();
            return Ok(tags.clone());
        }
        Err(RepositoryError::NotFound(old_tag.to_string()))
    }
}

#[test]
fn list_tags_returns_all_tags() {
    let catalog = FakeTagCatalog::default();
    catalog.add_tag("work").unwrap();
    catalog.add_tag("daily").unwrap();

    let use_case = ListTagsUseCase::new(catalog);
    let tags = use_case.execute().expect("list should succeed");

    assert_eq!(tags, vec!["daily".to_string(), "work".to_string()]);
}

#[test]
fn add_tag_rejects_empty_values() {
    let use_case = AddTagUseCase::new(FakeTagCatalog::default());
    let result = use_case.execute("   ");

    assert_eq!(result, Err(AddTagError::InvalidTag));
}

#[test]
fn remove_tag_rejects_empty_values() {
    let use_case = RemoveTagUseCase::new(FakeTagCatalog::default());
    let result = use_case.execute(" ");

    assert_eq!(result, Err(RemoveTagError::InvalidTag));
}

#[test]
fn update_tag_rejects_empty_values() {
    let use_case = UpdateTagUseCase::new(FakeTagCatalog::default());
    let result = use_case.execute("work", " ");

    assert_eq!(result, Err(UpdateTagError::InvalidTag));
}

#[test]
fn update_tag_renames_existing_tag() {
    let catalog = FakeTagCatalog::default();
    catalog.add_tag("todo").unwrap();

    let use_case = UpdateTagUseCase::new(catalog);
    let result = use_case
        .execute("todo", "next")
        .expect("update should succeed");

    assert_eq!(result, vec!["next".to_string()]);
}
