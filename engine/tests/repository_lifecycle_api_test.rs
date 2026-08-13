use git2::Repository;
use penna_engine::{CloneJournalRequest, CreateEntryApiRequest, PennaEngine};
use tempfile::TempDir;

#[test]
fn clone_journal_creates_session_and_resolves_path() {
    let source_dir = TempDir::new().expect("source temp dir should exist");
    let source_engine = PennaEngine::new();
    let source_session = source_engine
        .connect_journal(source_dir.path())
        .expect("source connect should succeed");

    source_engine
        .create_entry_api(CreateEntryApiRequest {
            session_id: source_session.session_id,
            title: "Seed".to_string(),
            body: "Seed body".to_string(),
            tags: vec![],
        })
        .expect("source entry should be created");

    let clone_parent = TempDir::new().expect("clone parent should exist");
    let engine = PennaEngine::new();

    let session = engine
        .clone_journal(CloneJournalRequest {
            remote_url: source_dir.path().to_string_lossy().to_string(),
            local_parent_dir: clone_parent.path().to_string_lossy().to_string(),
            directory_name: "my-journal".to_string(),
        })
        .expect("clone should succeed");

    let resolved = engine
        .resolve_journal_path(&session.session_id)
        .expect("resolve path should succeed");

    assert!(resolved.repo_path.ends_with("my-journal"));
    assert!(std::path::Path::new(&resolved.repo_path).join(".git").exists());
}

#[test]
fn push_journal_pushes_local_changes_to_remote() {
    let remote_dir = TempDir::new().expect("remote dir should exist");
    Repository::init_bare(remote_dir.path()).expect("bare remote should init");

    let local_dir = TempDir::new().expect("local dir should exist");
    let engine = PennaEngine::new();
    let session = engine
        .connect_journal(local_dir.path())
        .expect("connect should succeed");

    {
        let repo = Repository::open(local_dir.path()).expect("local repo should open");
        repo.remote("origin", remote_dir.path().to_str().expect("remote path should be utf8"))
            .expect("origin should be added");
    }

    engine
        .create_entry_api(CreateEntryApiRequest {
            session_id: session.session_id.clone(),
            title: "Push me".to_string(),
            body: "Body".to_string(),
            tags: vec![],
        })
        .expect("entry should be created");

    let result = engine
        .push_journal(&session.session_id)
        .expect("push should succeed");

    assert_eq!(result.status, "pushed");
}

#[test]
fn pull_journal_pulls_remote_changes() {
    let remote_dir = TempDir::new().expect("remote dir should exist");
    Repository::init_bare(remote_dir.path()).expect("bare remote should init");

    let local_dir_a = TempDir::new().expect("local A should exist");
    let engine_a = PennaEngine::new();
    let session_a = engine_a
        .connect_journal(local_dir_a.path())
        .expect("connect A should succeed");

    {
        let repo = Repository::open(local_dir_a.path()).expect("local A repo should open");
        repo.remote("origin", remote_dir.path().to_str().expect("remote path should be utf8"))
            .expect("origin should be added");
    }

    let first = engine_a
        .create_entry_api(CreateEntryApiRequest {
            session_id: session_a.session_id.clone(),
            title: "First".to_string(),
            body: "Body".to_string(),
            tags: vec![],
        })
        .expect("first entry should be created");
    assert!(!first.id.is_empty());

    engine_a
        .push_journal(&session_a.session_id)
        .expect("initial push should succeed");

    let local_dir_b = TempDir::new().expect("local B should exist");
    Repository::clone(
        remote_dir.path().to_str().expect("remote path should be utf8"),
        local_dir_b.path(),
    )
    .expect("clone B should succeed");

    let engine_b = PennaEngine::new();
    let session_b = engine_b
        .connect_journal(local_dir_b.path())
        .expect("connect B should succeed");

    let second = engine_a
        .create_entry_api(CreateEntryApiRequest {
            session_id: session_a.session_id.clone(),
            title: "Second".to_string(),
            body: "Body".to_string(),
            tags: vec![],
        })
        .expect("second entry should be created");

    engine_a
        .push_journal(&session_a.session_id)
        .expect("second push should succeed");

    let pull_result = engine_b
        .pull_journal(&session_b.session_id)
        .expect("pull should succeed");

    assert_eq!(pull_result.status, "pulled");

    let loaded = engine_b
        .get_entry(&session_b.session_id, &second.id)
        .expect("get should succeed");
    assert!(loaded.is_some());
}
