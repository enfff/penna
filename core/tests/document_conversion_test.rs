use std::error::Error;

use penna_core::application::{
    DocumentToMarkdownUseCase, DocumentWithSidecarInput, DocumentWithSidecarUseCase,
    MarkdownToDocumentInput, MarkdownToDocumentUseCase,
};
use penna_core::domain::{Document, Node, Sidecar};
use penna_core::ports::{MarkdownExporter, MarkdownImporter};

struct FakeMarkdownImporter;

impl MarkdownImporter for FakeMarkdownImporter {
    fn import(&self, markdown: &str, _frontmatter: &str) -> Result<Document, Box<dyn Error + Send + Sync>> {
        Ok(Document {
            content: vec![Node {
                node_type: "paragraph".to_string(),
                content: None,
                marks: None,
                text: Some(markdown.to_string()),
                attrs: None,
            }],
            schema_version: 1,
        })
    }
}

struct FakeMarkdownExporter;

impl MarkdownExporter for FakeMarkdownExporter {
    fn export(&self, document: &Document) -> Result<String, Box<dyn Error + Send + Sync>> {
        Ok(document
            .content
            .iter()
            .filter_map(|n| n.text.clone())
            .collect::<Vec<_>>()
            .join("\n"))
    }

    fn export_with_sidecar(
        &self,
        document: &Document,
        sidecar: Option<&Sidecar>,
        include_sidecar: bool,
    ) -> Result<(String, Option<String>), Box<dyn Error + Send + Sync>> {
        let markdown = self.export(document)?;
        
        if include_sidecar
            && let Some(sc) = sidecar
        {
            return Ok((markdown, Some(serde_json::to_string(sc)?)));
        }
        
        Ok((markdown, None))
    }
}

#[test]
fn markdown_to_document_imports_content() {
    let importer = FakeMarkdownImporter;
    let use_case = MarkdownToDocumentUseCase::new(importer);

    let input = MarkdownToDocumentInput {
        markdown_body: "# Hello World\n\nThis is a test.".to_string(),
        frontmatter: None,
    };

    let document = use_case
        .execute(input)
        .expect("import should succeed");

    assert_eq!(document.schema_version, 1);
    assert!(!document.content.is_empty());
}

#[test]
fn markdown_to_document_handles_frontmatter() {
    let importer = FakeMarkdownImporter;
    let use_case = MarkdownToDocumentUseCase::new(importer);

    let input = MarkdownToDocumentInput {
        markdown_body: "Test content".to_string(),
        frontmatter: Some("title: Test".to_string()),
    };

    let document = use_case
        .execute(input)
        .expect("import should succeed");

    assert!(!document.content.is_empty());
}

#[test]
fn document_to_markdown_exports_content() {
    let exporter = FakeMarkdownExporter;
    let use_case = DocumentToMarkdownUseCase::new(exporter);

    let document = Document {
        content: vec![Node {
            node_type: "paragraph".to_string(),
            content: None,
            marks: None,
            text: Some("Hello World".to_string()),
            attrs: None,
        }],
        schema_version: 1,
    };

    let markdown = use_case
        .execute(&document)
        .expect("export should succeed");

    assert!(markdown.contains("Hello World"));
}

#[test]
fn document_with_sidecar_exports_without_sidecar() {
    let exporter = FakeMarkdownExporter;
    let use_case = DocumentWithSidecarUseCase::new(exporter);

    let document = Document {
        content: vec![Node {
            node_type: "paragraph".to_string(),
            content: None,
            marks: None,
            text: Some("Test content".to_string()),
            attrs: None,
        }],
        schema_version: 1,
    };

    let input = DocumentWithSidecarInput {
        document,
        sidecar: None,
        include_sidecar: false,
    };

    let (markdown, sidecar_json) = use_case
        .execute(input)
        .expect("export should succeed");

    assert!(markdown.contains("Test content"));
    assert!(sidecar_json.is_none());
}

#[test]
fn document_with_sidecar_exports_with_sidecar_when_included() {
    let exporter = FakeMarkdownExporter;
    let use_case = DocumentWithSidecarUseCase::new(exporter);

    let document = Document {
        content: vec![Node {
            node_type: "paragraph".to_string(),
            content: None,
            marks: None,
            text: Some("Test content".to_string()),
            attrs: None,
        }],
        schema_version: 1,
    };

    let sidecar = Sidecar {
        schema_version: 1,
        entry_id: "test-id".to_string(),
        generated_at: "2024-01-01T00:00:00Z".to_string(),
        blocks: vec![],
        attachments: None,
        revisions: None,
    };

    let input = DocumentWithSidecarInput {
        document,
        sidecar: Some(sidecar),
        include_sidecar: true,
    };

    let (markdown, sidecar_json) = use_case
        .execute(input)
        .expect("export should succeed");

    assert!(markdown.contains("Test content"));
    assert!(sidecar_json.is_some());
    assert!(sidecar_json.unwrap().contains("test-id"));
}

#[test]
fn document_with_sidecar_excludes_sidecar_when_flag_false() {
    let exporter = FakeMarkdownExporter;
    let use_case = DocumentWithSidecarUseCase::new(exporter);

    let document = Document {
        content: vec![Node {
            node_type: "paragraph".to_string(),
            content: None,
            marks: None,
            text: Some("Test content".to_string()),
            attrs: None,
        }],
        schema_version: 1,
    };

    let sidecar = Sidecar {
        schema_version: 1,
        entry_id: "test-id".to_string(),
        generated_at: "2024-01-01T00:00:00Z".to_string(),
        blocks: vec![],
        attachments: None,
        revisions: None,
    };

    let input = DocumentWithSidecarInput {
        document,
        sidecar: Some(sidecar),
        include_sidecar: false,
    };

    let (markdown, sidecar_json) = use_case
        .execute(input)
        .expect("export should succeed");

    assert!(markdown.contains("Test content"));
    assert!(sidecar_json.is_none());
}
