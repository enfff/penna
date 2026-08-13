use penna_engine::{EngineError, EngineErrorDto};
use penna_core::application::{
    AddTagError, CreateEntryError, RemoveTagError, UpdateEntryError, UpdateTagError,
};
use penna_core::domain::DomainError;
use penna_core::ports::RepositoryError;

#[test]
fn maps_validation_errors_to_validation_code() {
    let create_error = EngineError::Create(CreateEntryError::Domain(DomainError::EmptyTitle));
    let update_error = EngineError::Update(UpdateEntryError::Domain(DomainError::EmptyTitle));
    let add_tag_error = EngineError::AddTag(AddTagError::InvalidTag);
    let remove_tag_error = EngineError::RemoveTag(RemoveTagError::InvalidTag);
    let update_tag_error = EngineError::UpdateTag(UpdateTagError::InvalidTag);

    assert_eq!(create_error.code(), "VALIDATION");
    assert_eq!(update_error.code(), "VALIDATION");
    assert_eq!(add_tag_error.code(), "VALIDATION");
    assert_eq!(remove_tag_error.code(), "VALIDATION");
    assert_eq!(update_tag_error.code(), "VALIDATION");
}

#[test]
fn maps_repo_related_errors_to_repo_code() {
    let repo_error = EngineError::Repo(RepositoryError::NotFound("missing".to_string()));
    let create_repo_error =
        EngineError::Create(CreateEntryError::Repository(RepositoryError::Storage("x".to_string())));

    assert_eq!(repo_error.code(), "REPO");
    assert_eq!(create_repo_error.code(), "REPO");
}

#[test]
fn converts_error_to_dto() {
    let error = EngineError::NotConnected("session-123".to_string());
    let dto: EngineErrorDto = error.to_dto();

    assert_eq!(dto.code, "NOT_CONNECTED");
    assert_eq!(dto.message, "session-123");
}
