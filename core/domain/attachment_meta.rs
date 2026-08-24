use serde::{Deserialize, Serialize};

/// Manifest entry describing one stored attachment (ADR 0012).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AttachmentMeta {
    pub name: String,
    pub bytes: u64,
}
