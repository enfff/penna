use penna_adapters_markdown::{MarkdownExporter, MarkdownImporter};
use penna_core::domain::Sidecar;
use penna_core::ports::{
    MarkdownExporter as MarkdownExporterPort, MarkdownImporter as MarkdownImporterPort,
};

#[test]
fn preserves_unknown_frontmatter_and_strips_sidecar_by_default() {
    let importer = MarkdownImporter;
    let exporter = MarkdownExporter;

    let frontmatter = r#"id: test-id
title: Test
custom_unknown: keep-me
penna_sidecar: sidecars/test-id.json"#;

    let document = MarkdownImporterPort::import(&importer, "Body line", frontmatter)
        .expect("import should succeed");

    let (markdown, sidecar_json) =
        MarkdownExporterPort::export_with_sidecar(&exporter, &document, None, false)
            .expect("export should succeed");

    assert!(markdown.contains("custom_unknown: keep-me"));
    assert!(!markdown.contains("penna_sidecar:"));
    assert!(markdown.contains("Body line"));
    assert!(sidecar_json.is_none());
}

#[test]
fn malformed_markdown_is_imported_without_failure() {
    let importer = MarkdownImporter;
    let exporter = MarkdownExporter;

    let malformed = "# Heading\n\n```\nunterminated block\n<<<<<";

    let document = MarkdownImporterPort::import(&importer, malformed, "")
        .expect("malformed markdown should still import");

    let markdown = MarkdownExporterPort::export(&exporter, &document)
        .expect("export should succeed");

    assert!(markdown.contains("unterminated block"));
    assert!(markdown.contains("<<<<<"));
}

#[test]
fn export_with_sidecar_includes_json_when_requested() {
    let importer = MarkdownImporter;
    let exporter = MarkdownExporter;

    let document = MarkdownImporterPort::import(&importer, "Body", "")
        .expect("import should succeed");

    let sidecar = Sidecar {
        schema_version: 1,
        entry_id: "test-id".to_string(),
        generated_at: "2026-08-09T00:00:00Z".to_string(),
        blocks: vec![],
        attachments: None,
        revisions: None,
    };

    let (_, sidecar_json) =
        MarkdownExporterPort::export_with_sidecar(&exporter, &document, Some(&sidecar), true)
            .expect("export should succeed");

    let sidecar_json = sidecar_json.expect("sidecar should be included");
    assert!(sidecar_json.contains("\"entry_id\":\"test-id\""));
}
