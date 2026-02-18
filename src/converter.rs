use crate::cli::Cli;
use crate::epub_reader::EpubData;
use crate::image::{self, ImageMap};
use crate::markdown;
use crate::metadata;
use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

struct ConvertedChapter {
    title: String,
    filename: String,
    content: String,
}

pub fn convert(cli: &Cli) -> Result<()> {
    let epub = EpubData::open(&cli.input)?;
    let output_path = resolve_output_path(cli)?;
    let metadata_header = metadata::format_metadata(&epub);

    // Resolve the images output dir:
    // - Folder mode: images go inside the output directory
    // - Single mode: images go next to the output file
    let images_base = if cli.single {
        output_path
            .parent()
            .unwrap_or_else(|| std::path::Path::new("."))
            .to_path_buf()
    } else {
        output_path.clone()
    };

    // Extract images unless --no-images
    let image_map = if !cli.no_images {
        // Ensure the base dir exists before extracting images
        fs::create_dir_all(&images_base)?;
        image::extract_images(&epub, &images_base)?
    } else {
        ImageMap::new()
    };

    // Convert chapters
    let raw_chapters = epub.chapters()?;
    let mut converted = Vec::new();

    for (i, chapter) in raw_chapters.iter().enumerate() {
        let md_content = markdown::html_to_markdown(&chapter.html_content, &image_map);

        // Try to extract title from the converted markdown (first # heading)
        let title = chapter
            .title
            .clone()
            .or_else(|| extract_title_from_markdown(&md_content))
            .unwrap_or_else(|| format!("Chapter {}", i + 1));

        let filename = format!("chapter-{:02}.md", i + 1);

        converted.push(ConvertedChapter {
            title,
            filename,
            content: md_content,
        });
    }

    if cli.single {
        write_single_file(&output_path, &metadata_header, &converted)?;
    } else {
        write_folder(&output_path, &metadata_header, &converted)?;
    }

    let chapter_count = converted.len();
    let image_count = image_map.len();
    eprintln!(
        "Converted {} chapters{} to {}",
        chapter_count,
        if image_count > 0 {
            format!(" and {} images", image_count)
        } else {
            String::new()
        },
        output_path.display()
    );

    Ok(())
}

fn resolve_output_path(cli: &Cli) -> Result<PathBuf> {
    if let Some(ref path) = cli.output {
        return Ok(path.clone());
    }

    let stem = cli
        .input
        .file_stem()
        .context("Input file has no name")?
        .to_string_lossy();

    if cli.single {
        Ok(PathBuf::from(format!("{}.md", stem)))
    } else {
        Ok(PathBuf::from(stem.as_ref()))
    }
}

fn extract_title_from_markdown(md: &str) -> Option<String> {
    for line in md.lines() {
        let trimmed = line.trim();
        if let Some(title) = trimmed.strip_prefix("# ") {
            let title = title.trim();
            if !title.is_empty() {
                return Some(title.to_string());
            }
        }
    }
    None
}

fn write_single_file(
    output_path: &PathBuf,
    metadata_header: &str,
    chapters: &[ConvertedChapter],
) -> Result<()> {
    let mut content = String::new();

    content.push_str(metadata_header);

    for (i, chapter) in chapters.iter().enumerate() {
        if i > 0 {
            content.push_str("\n---\n\n");
        }
        content.push_str(&chapter.content);
        content.push('\n');
    }

    if let Some(parent) = output_path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }

    fs::write(output_path, &content)
        .with_context(|| format!("Failed to write output file: {}", output_path.display()))?;

    Ok(())
}

fn write_folder(
    output_dir: &PathBuf,
    metadata_header: &str,
    chapters: &[ConvertedChapter],
) -> Result<()> {
    fs::create_dir_all(output_dir)?;

    // Write chapter files
    for chapter in chapters {
        let path = output_dir.join(&chapter.filename);
        fs::write(&path, &chapter.content)
            .with_context(|| format!("Failed to write chapter: {}", path.display()))?;
    }

    // Write README.md with metadata and table of contents
    let mut readme = String::new();
    readme.push_str(metadata_header);
    readme.push_str("## Table of Contents\n\n");

    for (i, chapter) in chapters.iter().enumerate() {
        readme.push_str(&format!(
            "{}. [{}]({})\n",
            i + 1,
            chapter.title,
            chapter.filename
        ));
    }

    readme.push('\n');

    fs::write(output_dir.join("README.md"), &readme)
        .with_context(|| "Failed to write README.md")?;

    Ok(())
}
