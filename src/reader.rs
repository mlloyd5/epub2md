use anyhow::Result;

/// Shared chapter representation across all input formats
pub struct Chapter {
    pub title: Option<String>,
    /// Already-converted markdown content
    pub content: String,
}

/// Shared image representation across all input formats
pub struct ImageResource {
    pub original_href: String,
    pub data: Vec<u8>,
}

/// Shared metadata representation across all input formats
pub struct Metadata {
    pub title: Option<String>,
    pub authors: Vec<String>,
    pub publisher: Option<String>,
    pub language: Option<String>,
    pub description: Option<String>,
}

/// Trait for reading document formats (EPUB, DOCX, etc.)
pub trait BookReader {
    /// Extract chapters as markdown content
    fn chapters(&self) -> Result<Vec<Chapter>>;
    /// Extract embedded images
    fn images(&self) -> Result<Vec<ImageResource>>;
    /// Extract document metadata
    fn metadata(&self) -> Metadata;
}
