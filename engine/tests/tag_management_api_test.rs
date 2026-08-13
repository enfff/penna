use penna_engine::PennaEngine;
use tempfile::TempDir;

#[test]
fn tag_catalog_crud_roundtrip() {
    let temp_dir = TempDir::new().expect("temp dir should exist");
    let engine = PennaEngine::new();

    let session = engine
        .connect_journal(temp_dir.path())
        .expect("connect should succeed");

    let tags = engine
        .add_tag(&session.session_id, "work")
        .expect("add tag should succeed");
    assert_eq!(tags, vec!["work".to_string()]);

    let tags = engine
        .add_tag(&session.session_id, "daily")
        .expect("add second tag should succeed");
    assert_eq!(tags, vec!["daily".to_string(), "work".to_string()]);

    let tags = engine
        .update_tag(&session.session_id, "daily", "journal")
        .expect("update tag should succeed");
    assert_eq!(tags, vec!["journal".to_string(), "work".to_string()]);

    let tags = engine
        .remove_tag(&session.session_id, "work")
        .expect("remove tag should succeed");
    assert_eq!(tags, vec!["journal".to_string()]);

    let listed = engine
        .list_tags(&session.session_id)
        .expect("list tags should succeed");
    assert_eq!(listed, vec!["journal".to_string()]);
}

#[test]
fn add_tag_rejects_empty_value() {
    let temp_dir = TempDir::new().expect("temp dir should exist");
    let engine = PennaEngine::new();

    let session = engine
        .connect_journal(temp_dir.path())
        .expect("connect should succeed");

    let error = engine
        .add_tag(&session.session_id, "   ")
        .expect_err("empty tag should fail");

    assert_eq!(error.code(), "VALIDATION");
}
