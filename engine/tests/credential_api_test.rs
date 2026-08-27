use penna_engine::{EngineError, PennaEngine};
use std::sync::{Mutex, MutexGuard};

/// The keyring dbus backend keeps a process-wide item cache; concurrent
/// store/delete calls from test threads observe stale item paths. Serialize
/// all keychain-touching tests.
fn keychain_lock() -> MutexGuard<'static, ()> {
    static LOCK: Mutex<()> = Mutex::new(());
    LOCK.lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// Unique remote URL per test so parallel test threads never collide on a
/// keychain entry, and no run can touch a real user credential.
fn probe_remote(tag: &str) -> String {
    format!(
        "https://keychain-probe.invalid/penna-test/{}-{}.git",
        std::process::id(),
        tag
    )
}

#[test]
fn store_credential_rejects_blank_secret_without_touching_keychain() {
    let engine = PennaEngine::new();
    let url = probe_remote("blank");

    for blank in ["", "   ", "\t\n"] {
        let error = engine
            .store_credential(&url, blank)
            .expect_err("blank secret must be rejected");
        assert!(
            matches!(error, EngineError::Validation(_)),
            "blank secret must map to Validation, got {error:?}"
        );
        assert_eq!(error.code(), "VALIDATION");
    }
}

#[test]
fn store_lookup_delete_roundtrip() {
    let _guard = keychain_lock();
    let engine = PennaEngine::new();
    let url = probe_remote("roundtrip");

    // Probe whether this environment has a usable secret store (headless
    // CI has no session bus; skip the roundtrip there rather than fail).
    if engine.store_credential(&url, "probe-token").is_err() {
        eprintln!("secret store unavailable in this environment; skipping keychain roundtrip");
        return;
    }

    assert!(
        engine.has_credential(&url),
        "stored credential must be reported present"
    );

    engine
        .delete_credential(&url)
        .expect("delete of a stored credential must succeed");
    assert!(
        !engine.has_credential(&url),
        "deleted credential must be reported absent"
    );
}

#[test]
fn has_credential_is_false_for_unknown_remote_when_keychain_present() {
    let _guard = keychain_lock();
    let engine = PennaEngine::new();
    let url = probe_remote("unknown");

    if engine.store_credential(&url, "probe-token").is_err() {
        eprintln!("secret store unavailable in this environment; skipping");
        return;
    }

    let never_stored = probe_remote("never-stored");
    assert!(!engine.has_credential(&never_stored));

    engine
        .delete_credential(&url)
        .expect("cleanup must succeed");
}
