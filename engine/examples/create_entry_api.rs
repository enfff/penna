use penna_engine::{CreateEntryApiRequest, PennaEngine};

fn main() {
    let repo_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "/home/enf/Projects/penna-myjournal".to_string());

    let title = std::env::args()
        .nth(2)
        .unwrap_or_else(|| "Test entry from engine API".to_string());

    let body = std::env::args()
        .nth(3)
        .unwrap_or_else(|| "Plain markdown body".to_string());

    let engine = PennaEngine::new();

    let session = engine
        .connect_journal(&repo_path)
        .expect("connect_journal failed");

    let created = engine
        .create_entry_api(CreateEntryApiRequest {
            session_id: session.session_id.clone(),
            title,
            body,
            tags: vec!["manual-test".to_string()],
        })
        .expect("create_entry_api failed");

    println!(
        "{}",
        serde_json::to_string_pretty(&created).expect("serialize created entry failed")
    );

    engine
        .disconnect_journal(&session.session_id)
        .expect("disconnect_journal failed");
}
