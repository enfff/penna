use penna_engine::{EngineError, EngineErrorDto, PennaEngine, PUBLIC_ERROR_CODES};
use penna_core::application::{
    AddTagError, CreateEntryError, RemoveTagError, UpdateEntryError, UpdateTagError,
};
use penna_core::domain::DomainError;
use penna_core::ports::RepositoryError;

/// Constructs one representative error for every `EngineError` variant so
/// the taxonomy tests cannot silently skip newly added cases.
fn all_variants() -> Vec<(&'static str, EngineError)> {
    vec![
        ("Io", EngineError::Io("disk failure".to_string())),
        (
            "NotConnected",
            EngineError::NotConnected("session-123".to_string()),
        ),
        (
            "Repo",
            EngineError::Repo(RepositoryError::Storage("corrupt".to_string())),
        ),
        (
            "Create",
            EngineError::Create(CreateEntryError::Domain(DomainError::EmptyTitle)),
        ),
        (
            "Update",
            EngineError::Update(UpdateEntryError::Domain(DomainError::EmptyTitle)),
        ),
        ("AddTag", EngineError::AddTag(AddTagError::InvalidTag)),
        ("RemoveTag", EngineError::RemoveTag(RemoveTagError::InvalidTag)),
        ("UpdateTag", EngineError::UpdateTag(UpdateTagError::InvalidTag)),
        (
            "IdCollision",
            EngineError::IdCollision("no free slot".to_string()),
        ),
    ]
}

#[test]
fn public_error_codes_are_exactly_the_documented_five() {
    let mut sorted = PUBLIC_ERROR_CODES.to_vec();
    sorted.sort();
    assert_eq!(
        sorted,
        vec!["CONFLICT", "IO", "NOT_CONNECTED", "REPO", "VALIDATION"]
    );
}

#[test]
fn every_variant_maps_inside_the_closed_code_set() {
    for (variant, error) in all_variants() {
        let code = error.code();
        assert!(
            PUBLIC_ERROR_CODES.contains(&code),
            "variant {} maps to code {} which is outside the closed set (ADR 0009)",
            variant,
            code
        );
    }
}

#[test]
fn classification_matches_adr_0009_buckets() {
    let expected: Vec<(&str, &str)> = vec![
        ("Io", "IO"),
        ("NotConnected", "NOT_CONNECTED"),
        ("Repo", "REPO"),
        ("Create", "VALIDATION"),
        ("Update", "VALIDATION"),
        ("AddTag", "VALIDATION"),
        ("RemoveTag", "VALIDATION"),
        ("UpdateTag", "VALIDATION"),
        ("IdCollision", "CONFLICT"),
    ];

    for (variant, error) in all_variants() {
        let code = error.code();
        let bucket = expected
            .iter()
            .find(|(name, _)| *name == variant)
            .map(|(_, bucket)| *bucket)
            .unwrap_or_else(|| panic!("variant {} missing from expectation table", variant));
        assert_eq!(code, bucket, "variant {} misclassified", variant);
    }
}

#[test]
fn repository_wrappers_all_map_to_repo_code() {
    let repo = RepositoryError::NotFound("missing".to_string());
    let storage = RepositoryError::Storage("io".to_string());

    assert_eq!(EngineError::Repo(repo.clone()).code(), "REPO");
    assert_eq!(
        EngineError::Create(CreateEntryError::Repository(repo.clone())).code(),
        "REPO"
    );
    assert_eq!(
        EngineError::Update(UpdateEntryError::Repository(repo.clone())).code(),
        "REPO"
    );
    assert_eq!(EngineError::AddTag(AddTagError::Repository(repo.clone())).code(), "REPO");
    assert_eq!(
        EngineError::RemoveTag(RemoveTagError::Repository(repo.clone())).code(),
        "REPO"
    );
    assert_eq!(EngineError::UpdateTag(UpdateTagError::Repository(storage)).code(), "REPO");
}

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

#[test]
fn stale_session_maps_to_not_connected_code() {
    let engine = PennaEngine::new();
    let error = engine
        .journal_status("session-does-not-exist")
        .expect_err("unknown session must fail");

    assert_eq!(error.code(), "NOT_CONNECTED");
    let dto = error.to_dto();
    assert_eq!(dto.code, "NOT_CONNECTED");
}
