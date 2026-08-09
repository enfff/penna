use penna_engine::PennaEngine;

#[test]
fn reports_missing_sidecar() {
    let engine = PennaEngine::new();

    let report = engine.sidecar_integrity_status("202608091300", None);

    assert_eq!(report.status, "missing");
    assert!(report.reason.is_none());
}

#[test]
fn reports_ok_when_sidecar_entry_id_matches() {
    let engine = PennaEngine::new();

    let sidecar = r#"{
      "schema_version": 1,
      "entry_id": "202608091300",
      "generated_at": "2026-08-09T13:00:00Z",
      "blocks": [],
      "attachments": null,
      "revisions": null
    }"#;

    let report = engine.sidecar_integrity_status("202608091300", Some(sidecar));

    assert_eq!(report.status, "ok");
}

#[test]
fn reports_mismatch_when_sidecar_entry_id_differs() {
    let engine = PennaEngine::new();

    let sidecar = r#"{
      "schema_version": 1,
      "entry_id": "different-id",
      "generated_at": "2026-08-09T13:00:00Z",
      "blocks": [],
      "attachments": null,
      "revisions": null
    }"#;

    let report = engine.sidecar_integrity_status("202608091300", Some(sidecar));

    assert_eq!(report.status, "mismatch");
    assert_eq!(report.expected_entry_id.as_deref(), Some("202608091300"));
    assert_eq!(report.actual_entry_id.as_deref(), Some("different-id"));
}

#[test]
fn reports_malformed_when_sidecar_json_is_invalid() {
    let engine = PennaEngine::new();

    let report = engine.sidecar_integrity_status("202608091300", Some("{invalid"));

    assert_eq!(report.status, "malformed");
    assert!(report.reason.is_some());
}
