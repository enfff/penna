use penna_engine::{CreateEntryRequest, PennaEngine};
use tempfile::TempDir;

#[test]
fn connect_repo_and_create_entry_with_timestamp_id_format() {
    let temp_dir = TempDir::new().expect("temp dir should exist");
    let engine = PennaEngine::new();

    let session = engine
        .connect_journal(temp_dir.path())
        .expect("connect should succeed");

    let status = engine
        .journal_status(&session.session_id)
        .expect("status should succeed");

    assert_eq!(status.repo_path, temp_dir.path().to_string_lossy());

    let entry = engine
        .create_entry(
            &session.session_id,
            CreateEntryRequest {
                title: "Hello".to_string(),
                body: "Plain markdown body".to_string(),
                tags: vec!["test".to_string()],
            },
        )
        .expect("create should succeed");

    let second_entry = engine
        .create_entry(
            &session.session_id,
            CreateEntryRequest {
                title: "Hello Again".to_string(),
                body: "Second plain markdown body".to_string(),
                tags: vec!["test".to_string()],
            },
        )
        .expect("second create should succeed even in same minute");

    // YYYYMMDDHHmm format.
    assert_eq!(entry.id.0.len(), 12);
    assert!(entry.id.0.chars().all(|c| c.is_ascii_digit()));
    assert_eq!(second_entry.id.0.len(), 12);
    assert_ne!(entry.id.0, second_entry.id.0);

    let loaded = engine
        .get_entry(&session.session_id, &entry.id.0)
        .expect("get should succeed")
        .expect("entry should exist");

    assert_eq!(loaded.title, "Hello");

    engine
        .disconnect_journal(&session.session_id)
        .expect("disconnect should succeed");
}
