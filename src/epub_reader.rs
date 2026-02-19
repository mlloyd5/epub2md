use crate::image::ImageMap;
use crate::markdown;
use crate::reader::{BookReader, Chapter, ImageResource, Metadata};
use anyhow::{Context, Result};
use rbook::prelude::*;
use rbook::Epub;
use std::path::Path;

pub struct EpubData {
    epub: Epub,
}

impl EpubData {
    pub fn open(path: &Path) -> Result<Self> {
        let epub = Epub::options()
            .strict(false)
            .open(path)
            .with_context(|| format!("Failed to open EPUB: {}", path.display()))?;
        Ok(Self { epub })
    }

    fn raw_chapters(&self) -> Result<Vec<RawChapter>> {
        let mut chapters = Vec::new();
        let mut reader = self.epub.reader();

        while let Some(result) = reader.read_next() {
            let data = result.context("Failed to read chapter content")?;
            let html_content = data.content().to_string();

            // Skip empty or near-empty content
            if html_content.trim().is_empty() {
                continue;
            }

            chapters.push(RawChapter {
                title: None,
                html_content,
            });
        }

        Ok(chapters)
    }

    /// Convert raw HTML chapters to markdown with image path rewriting
    pub fn convert_chapters(&self, image_map: &ImageMap) -> Result<Vec<Chapter>> {
        let raw = self.raw_chapters()?;
        let mut chapters = Vec::new();

        for raw_ch in &raw {
            let md_content = markdown::html_to_markdown(&raw_ch.html_content, image_map);
            chapters.push(Chapter {
                title: raw_ch.title.clone(),
                content: md_content,
            });
        }

        Ok(chapters)
    }
}

impl BookReader for EpubData {
    fn chapters(&self) -> Result<Vec<Chapter>> {
        // When called without an image map, use an empty one
        self.convert_chapters(&ImageMap::new())
    }

    fn images(&self) -> Result<Vec<ImageResource>> {
        let mut images = Vec::new();
        for entry in self.epub.manifest().images() {
            let href = entry
                .resource()
                .key()
                .value()
                .unwrap_or("unknown")
                .to_string();

            let bytes = entry
                .read_bytes()
                .with_context(|| format!("Failed to read image: {}", href))?;

            images.push(ImageResource {
                original_href: href,
                data: bytes,
            });
        }

        Ok(images)
    }

    fn metadata(&self) -> Metadata {
        use rbook::prelude::Metadata as RbookMetadata;
        let meta = self.epub.metadata();
        Metadata {
            title: RbookMetadata::title(&meta).map(|t| t.value().to_string()),
            authors: RbookMetadata::creators(&meta)
                .map(|c| c.value().to_string())
                .collect(),
            publisher: RbookMetadata::publishers(&meta)
                .next()
                .map(|p| p.value().to_string()),
            language: RbookMetadata::languages(&meta)
                .next()
                .map(|l| l.value().to_string()),
            description: RbookMetadata::descriptions(&meta)
                .next()
                .map(|d| d.value().to_string()),
        }
    }
}

/// Internal raw chapter before markdown conversion
struct RawChapter {
    title: Option<String>,
    html_content: String,
}
