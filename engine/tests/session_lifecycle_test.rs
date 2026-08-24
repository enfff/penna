use penna_engine::{CreateEntryRequest, PennaEngine, UpdateEntryRequest};
use tempfile::TempDir;

fn connect(engine: &PennaEngine, path: &std::path::Path) -> penna_engine::JournalSession {
    engine
        .connect_journal(path)
        .expect("connect should succeed")
}

fn create_entry(engine: &PennaEngine, session_id: &str, title: &str) {
    engine
        .create_entry(
            session_id,
            CreateEntryRequest {
                title: title.to_string(),
                body: "body".to_string(),
                tags: vec![],
            },
        )
        .expect("entry creation should succeed");
}

#[test]
fn every_session_taking_api_surfaces_not_connected_for_unknown_ids() {
    let engine = PennaEngine::new();
    let ghost = "session-ghost";

    macro_rules! assert_not_connected {
        ($call:expr) => {
            let err = $call.expect_err("unknown session must fail");
            assert_eq!(err.code(), "NOT_CONNECTED", "unexpected error: {:?}", err);
        };
    }

    assert_not_connected!(engine.journal_status(ghost));
    assert_not_connected!(engine.disconnect_journal(ghost));
    assert_not_connected!(engine.resolve_journal_path(ghost));
    assert_not_connected!(engine.journal_status(ghost));
    assert_not_connected!(engine.list_entries(ghost));
    assert_not_connected!(engine.get_entry(ghost, "202608241200"));
    assert_not_connected!(engine.create_entry(
        ghost,
        CreateEntryRequest {
            title: "t".to_string(),
            body: "b".to_string(),
            tags: vec![]
        }
    ));
    assert_not_connected!(engine.update_entry(
        ghost,
        UpdateEntryRequest {
            id: "202608241200".to_string(),
            title: "t".to_string(),
            body: "b".to_string(),
            tags: vec![]
        }
    ));
    assert_not_connected!(engine.delete_entry(ghost, "202608241200"));
    assert_not_connected!(engine.sync_journal(ghost));
    assert_not_connected!(engine.pull_journal(ghost));
    assert_not_connected!(engine.push_journal(ghost));
    assert_not_connected!(engine.list_tags(ghost));
    assert_not_connected!(engine.add_tag(ghost, "work"));
    assert_not_connected!(engine.remove_tag(ghost, "work"));
    assert_not_connected!(engine.update_tag(ghost, "a", "b"));
}

#[test]
fn restart_invalidates_all_session_ids_and_reconnect_is_idempotent() {
    let temp_dir = TempDir::new().expect("temp dir should exist");

    let first_process = PennaEngine::new();
    let old_session = connect(&first_process, temp_dir.path());
    create_entry(&first_process, &old_session.session_id, "Survives restarts");

    drop(first_process);

    let second_process = PennaEngine::new();
    let err = second_process
        .journal_status(&old_session.session_id)
        .expect_err("old session id must be invalid after restart");
    assert_eq!(err.code(), "NOT_CONNECTED");

    let reconnected = connect(&second_process, temp_dir.path());
    assert_ne!(
        reconnected.session_id, old_session.session_id,
        "reconnect must mint a fresh opaque id"
    );

    let entries = second_process
        .list_entries(&reconnected.session_id)
        .expect("data survives process boundary");
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].title, "Survives restarts");
}

#[test]
fn disconnect_drops_only_the_handle_and_never_touches_data() {
    let temp_dir = TempDir::new().expect("temp dir should exist");
    let engine = PennaEngine::new();
    let session = connect(&engine, temp_dir.path());

    create_entry(&engine, &session.session_id, "Persistent note");
    engine
        .disconnect_journal(&session.session_id)
        .expect("disconnect should succeed");

    let entry_files: Vec<_> = std::fs::read_dir(temp_dir.path())
        .expect("journal directory must still exist")
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.extension().map_or(false, |ext| ext == "md"))
        .collect();
    assert_eq!(
        entry_files.len(),
        1,
        "working tree must keep the entry file"
    );

    let git_dir = temp_dir.path().join(".git");
    assert!(git_dir.exists(), "repository must stay intact");

    let double_disconnect = engine
        .disconnect_journal(&session.session_id)
        .expect_err("double disconnect is NOT_CONNECTED, not an error of user intent");
    assert_eq!(double_disconnect.code(), "NOT_CONNECTED");

    let fresh = connect(&engine, temp_dir.path());
    let status = engine
        .journal_status(&fresh.session_id)
        .expect("reconnect after disconnect works");
    assert_eq!(status.repo_path, temp_dir.path().to_string_lossy());
}

#[test]
fn concurrent_sessions_on_one_repository_are_permitted() {
    let temp_dir = TempDir::new().expect("temp dir should exist");
    let engine = PennaEngine::new();

    let session_a = connect(&engine, temp_dir.path());
    let session_b = connect(&engine, temp_dir.path());
    assert_ne!(session_a.session_id, session_b.session_id);

    create_entry(&engine, &session_a.session_id, "From session A");
    let seen_by_b = engine
        .list_entries(&session_b.session_id)
        .expect("session B reads what session A wrote");
    assert_eq!(seen_by_b.len(), 1);
    assert_eq!(seen_by_b[0].title, "From session A");

    create_entry(&engine, &session_b.session_id, "From session B");
    let seen_by_a = engine
        .list_entries(&session_a.session_id)
        .expect("session A reads what session B wrote");
    assert_eq!(seen_by_a.len(), 2);

    engine
        .add_tag(&session_a.session_id, "shared")
        .expect("tag catalog shared across sessions");
    let tags_b = engine
        .list_tags(&session_b.session_id)
        .expect("session B sees session A's tag");
    assert!(tags_b.contains(&"shared".to_string()));

    for session in [&session_a, &session_b] {
        engine
            .disconnect_journal(&session.session_id)
            .expect("each handle disconnects independently");
    }
}
