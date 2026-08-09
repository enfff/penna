use penna_core::domain::{Document, Node};
use penna_core::ports::MarkdownImporter as MarkdownImporterPort;
use serde_json::json;

pub struct MarkdownImporter;

impl MarkdownImporter {
    pub fn import(
        &self,
        markdown: &str,
        frontmatter: &str,
    ) -> Result<Document, Box<dyn std::error::Error + Send + Sync>> {
        // Always preserve input as text so malformed or unknown markdown never fails import.
        let attrs = if frontmatter.trim().is_empty() {
            None
        } else {
            Some(json!({ "frontmatter_raw": frontmatter }))
        };

        Ok(Document {
            content: vec![Node {
                node_type: "paragraph".to_string(),
                content: None,
                marks: None,
                text: Some(markdown.to_string()),
                attrs,
            }],
            schema_version: 1,
        })
    }
}

impl MarkdownImporterPort for MarkdownImporter {
    fn import(
        &self,
        markdown: &str,
        frontmatter: &str,
    ) -> Result<Document, Box<dyn std::error::Error + Send + Sync>> {
        Self::import(self, markdown, frontmatter)
    }
}
