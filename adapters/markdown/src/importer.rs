use penna_core::domain::Entry;

pub struct MarkdownImporter;

impl MarkdownImporter {
    pub fn import(&self, markdown: &str) -> Result<Entry, Box<dyn std::error::Error>> {
        unimplemented!()
    }
}
