use crate::docx_markdown;
use crate::image::ImageMap;
use crate::markdown;
use crate::reader::{BookReader, Chapter, ImageResource, Metadata};
use anyhow::{Context, Result};
use docx_rust::DocxFile;
use std::path::Path;

pub struct DocxData {
    /// DocxFile owns the raw data; Docx borrows from it.
    /// We store the file so it lives long enough, then parse on demand.
    file: DocxFile,
}

impl DocxData {
    pub fn open(path: &Path) -> Result<Self> {
        let file = DocxFile::from_file(path)
            .map_err(|e| anyhow::anyhow!("{}", e))
            .with_context(|| format!("Failed to open DOCX: {}", path.display()))?;
        Ok(Self { file })
    }

    fn parse(&self) -> Result<docx_rust::Docx<'_>> {
        self.file
            .parse()
            .map_err(|e| anyhow::anyhow!("{}", e))
            .context("Failed to parse DOCX content")
    }
}

impl BookReader for DocxData {
    fn chapters(&self) -> Result<Vec<Chapter>> {
        let docx = self.parse()?;

        // Convert the DOCX body to markdown (using empty image map — images already extracted)
        let md = docx_markdown::docx_to_markdown(&docx, &ImageMap::new());
        let cleaned = markdown::clean_markdown(&md);

        // DOCX is a single continuous document — treat as one chapter
        Ok(vec![Chapter {
            title: None,
            content: cleaned,
        }])
    }

    fn images(&self) -> Result<Vec<ImageResource>> {
        let docx = self.parse()?;
        let mut images = Vec::new();

        for (path, (_media_type, data)) in &docx.media {
            images.push(ImageResource {
                original_href: path.clone(),
                data: data.to_vec(),
            });
        }

        Ok(images)
    }

    fn metadata(&self) -> Metadata {
        let docx = match self.parse() {
            Ok(d) => d,
            Err(_) => {
                return Metadata {
                    title: None,
                    authors: Vec::new(),
                    publisher: None,
                    language: None,
                    description: None,
                }
            }
        };

        // Core is an enum with CoreNamespace and CoreNoNamespace variants
        // Both have the same fields, just different XML namespace handling
        let (title, creator, language, description) = match &docx.core {
            Some(docx_rust::core::Core::CoreNamespace(c)) => (
                c.title.as_deref().map(|s| s.to_string()),
                c.creator.as_deref().map(|s| s.to_string()),
                c.language.as_deref().map(|s| s.to_string()),
                c.description.as_deref().map(|s| s.to_string()),
            ),
            Some(docx_rust::core::Core::CoreNoNamespace(c)) => (
                c.title.as_deref().map(|s| s.to_string()),
                c.creator.as_deref().map(|s| s.to_string()),
                c.language.as_deref().map(|s| s.to_string()),
                c.description.as_deref().map(|s| s.to_string()),
            ),
            None => (None, None, None, None),
        };

        // App is also an enum with two namespace variants
        let company = match &docx.app {
            Some(docx_rust::app::App::AppNoApNamespace(a)) => {
                a.company.as_deref().map(|s| s.to_string())
            }
            Some(docx_rust::app::App::AppWithApNamespace(a)) => {
                a.company.as_deref().map(|s| s.to_string())
            }
            None => None,
        };

        Metadata {
            title,
            authors: creator.map(|a| vec![a]).unwrap_or_default(),
            publisher: company,
            language,
            description,
        }
    }
}
