use anyhow::{Context, Result};
use rbook::prelude::*;
use rbook::Epub;
use std::path::Path;

pub struct EpubData {
    epub: Epub,
}

pub struct Chapter {
    pub title: Option<String>,
    pub html_content: String,
}

pub struct ImageResource {
    pub original_href: String,
    pub data: Vec<u8>,
}

impl EpubData {
    pub fn open(path: &Path) -> Result<Self> {
        let epub = Epub::options()
            .strict(false)
            .open(path)
            .with_context(|| format!("Failed to open EPUB: {}", path.display()))?;
        Ok(Self { epub })
    }

    pub fn chapters(&self) -> Result<Vec<Chapter>> {
        let mut chapters = Vec::new();
        let mut reader = self.epub.reader();

        while let Some(result) = reader.read_next() {
            let data = result.context("Failed to read chapter content")?;
            let html_content = data.content().to_string();

            // Skip empty or near-empty content
            if html_content.trim().is_empty() {
                continue;
            }

            chapters.push(Chapter {
                title: None,
                html_content,
            });
        }

        Ok(chapters)
    }

    pub fn images(&self) -> Result<Vec<ImageResource>> {
        let mut images = Vec::new();
        for entry in self.epub.manifest().images() {
            // Get the resource path from the manifest entry
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

    pub fn title(&self) -> Option<String> {
        self.epub
            .metadata()
            .title()
            .map(|t| t.value().to_string())
    }

    pub fn authors(&self) -> Vec<String> {
        let mut authors = Vec::new();
        for creator in self.epub.metadata().creators() {
            authors.push(creator.value().to_string());
        }
        authors
    }

    pub fn language(&self) -> Option<String> {
        let mut langs = self.epub.metadata().languages();
        langs.next().map(|l| l.value().to_string())
    }

    pub fn description(&self) -> Option<String> {
        let mut descs = self.epub.metadata().descriptions();
        descs.next().map(|d| d.value().to_string())
    }

    pub fn publisher(&self) -> Option<String> {
        let mut pubs = self.epub.metadata().publishers();
        pubs.next().map(|p| p.value().to_string())
    }
}
