use penna_core::domain::{Document, Node, Sidecar};
use penna_core::ports::MarkdownExporter as MarkdownExporterPort;

pub struct MarkdownExporter;

impl MarkdownExporter {
    pub fn export(
        &self,
        document: &Document,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        Ok(render_markdown(document, None))
    }
}

impl MarkdownExporterPort for MarkdownExporter {
    fn export(
        &self,
        document: &Document,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        Self::export(self, document)
    }

    fn export_with_sidecar(
        &self,
        document: &Document,
        sidecar: Option<&Sidecar>,
        include_sidecar: bool,
    ) -> Result<(String, Option<String>), Box<dyn std::error::Error + Send + Sync>> {
        let frontmatter = extract_frontmatter_raw(document).map(|fm| {
            if include_sidecar {
                fm
            } else {
                strip_sidecar_pointer(&fm)
            }
        });

        let markdown = render_markdown(document, frontmatter.as_deref());
        let sidecar_json = if include_sidecar {
            sidecar
                .map(serde_json::to_string)
                .transpose()
                .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { Box::new(e) })?
        } else {
            None
        };

        Ok((markdown, sidecar_json))
    }
}

fn render_markdown(document: &Document, frontmatter: Option<&str>) -> String {
    let mut body = String::new();
    collect_text(&document.content, &mut body);

    if let Some(fm) = frontmatter {
        let fm = fm.trim();
        if !fm.is_empty() {
            return format!("---\n{fm}\n---\n\n{body}");
        }
    }

    body
}

fn collect_text(nodes: &[Node], out: &mut String) {
    for node in nodes {
        if let Some(text) = &node.text {
            if !out.is_empty() {
                out.push('\n');
            }
            out.push_str(text);
        }

        if let Some(children) = &node.content {
            collect_text(children, out);
        }
    }
}

fn extract_frontmatter_raw(document: &Document) -> Option<String> {
    for node in &document.content {
        if let Some(attrs) = &node.attrs
            && let Some(raw) = attrs.get("frontmatter_raw").and_then(|v| v.as_str())
        {
            return Some(raw.to_string());
        }
    }

    None
}

fn strip_sidecar_pointer(frontmatter: &str) -> String {
    frontmatter
        .lines()
        .filter(|line| !line.trim_start().starts_with("penna_sidecar:"))
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string()
}
