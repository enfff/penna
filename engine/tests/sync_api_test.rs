use git2::Repository;
use penna_engine::PennaEngine;
use tempfile::TempDir;

#[test]
fn sync_journal_reports_no_remote_for_local_repo_without_origin() {
    let temp_dir = TempDir::new().expect("temp dir should exist");
    let engine = PennaEngine::new();

    let session = engine
        .connect_journal(temp_dir.path())
        .expect("connect should succeed");

    let report = engine
        .sync_journal(&session.session_id)
        .expect("sync should return a report");

    assert_eq!(report.status, "no_remote");
}

#[test]
fn sync_journal_pushes_to_local_bare_remote() {
    let remote_dir = TempDir::new().expect("remote temp dir should exist");
    Repository::init_bare(remote_dir.path()).expect("bare repo should initialize");

    let local_dir = TempDir::new().expect("local temp dir should exist");
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
        .create_entry_api(penna_engine::CreateEntryApiRequest {
            session_id: session.session_id.clone(),
            title: "Sync Me".to_string(),
            body: "Body".to_string(),
            tags: vec![],
        })
        .expect("entry create should succeed");

    let report = engine
        .sync_journal(&session.session_id)
        .expect("sync should succeed");

    assert_eq!(report.status, "pushed");
    assert!(report.branch.is_some());
}
