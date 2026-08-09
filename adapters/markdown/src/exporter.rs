use penna_core::domain::Entry;

pub struct MarkdownExporter;

impl MarkdownExporter {
    pub fn export(&self, entry: &Entry) -> Result<String, Box<dyn std::error::Error>> {
        Ok(String::new())
    }
}
