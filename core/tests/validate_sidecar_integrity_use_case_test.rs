use penna_core::application::{
    SidecarIntegrityStatus, SidecarSource, ValidateSidecarIntegrityUseCase,
};
use penna_core::domain::Sidecar;

#[test]
fn returns_ok_when_entry_ids_match() {
    let use_case = ValidateSidecarIntegrityUseCase::new();
    let sidecar = Sidecar {
        schema_version: 1,
        entry_id: "202608091200".to_string(),
        generated_at: "2026-08-09T12:00:00Z".to_string(),
        blocks: vec![],
        attachments: None,
        revisions: None,
    };

    let status = use_case.execute("202608091200", SidecarSource::Present(sidecar));

    assert_eq!(status, SidecarIntegrityStatus::Ok);
}

#[test]
fn returns_missing_when_sidecar_is_missing() {
    let use_case = ValidateSidecarIntegrityUseCase::new();

    let status = use_case.execute("202608091200", SidecarSource::Missing);

    assert_eq!(status, SidecarIntegrityStatus::Missing);
}

#[test]
fn returns_mismatch_when_entry_ids_differ() {
    let use_case = ValidateSidecarIntegrityUseCase::new();
    let sidecar = Sidecar {
        schema_version: 1,
        entry_id: "different-id".to_string(),
        generated_at: "2026-08-09T12:00:00Z".to_string(),
        blocks: vec![],
        attachments: None,
        revisions: None,
    };

    let status = use_case.execute("202608091200", SidecarSource::Present(sidecar));

    assert_eq!(
        status,
        SidecarIntegrityStatus::Mismatch {
            expected_entry_id: "202608091200".to_string(),
            actual_entry_id: "different-id".to_string(),
        }
    );
}

#[test]
fn returns_malformed_when_sidecar_cannot_be_parsed() {
    let use_case = ValidateSidecarIntegrityUseCase::new();

    let status = use_case.execute(
        "202608091200",
        SidecarSource::Malformed("invalid JSON payload".to_string()),
    );

    assert_eq!(
        status,
        SidecarIntegrityStatus::Malformed {
            reason: "invalid JSON payload".to_string(),
        }
    );
}
