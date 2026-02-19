use crate::cli::Cli;
use crate::docx_reader::DocxData;
use crate::epub_reader::EpubData;
use crate::image::{self, ImageMap};
use crate::metadata;
use crate::reader::{BookReader, Chapter};
use anyhow::{bail, Context, Result};
use std::fs;
use std::path::PathBuf;

struct ConvertedChapter {
    title: String,
    filename: String,
    content: String,
}

pub fn convert(cli: &Cli) -> Result<()> {
    let ext = cli
        .input
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default();

    let output_path = resolve_output_path(cli)?;

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

    // Dispatch based on file extension
    match ext.as_str() {
        "epub" => convert_epub(cli, &output_path, &images_base),
        "docx" => convert_docx(cli, &output_path, &images_base),
        _ => bail!(
            "Unsupported file format: .{}. Supported formats: .epub, .docx",
            ext
        ),
    }
}

fn convert_epub(cli: &Cli, output_path: &PathBuf, images_base: &PathBuf) -> Result<()> {
    let epub = EpubData::open(&cli.input)?;
    let meta = epub.metadata();
    let metadata_header = metadata::format_metadata(&meta);

    // Extract images unless --no-images
    let image_map = if !cli.no_images {
        fs::create_dir_all(images_base)?;
        image::extract_images(&epub, images_base)?
    } else {
        ImageMap::new()
    };

    // EPUB needs image map for path rewriting during html→md conversion
    let chapters = epub.convert_chapters(&image_map)?;

    let converted = build_converted_chapters(&chapters)?;
    write_output(cli, output_path, &metadata_header, &converted)?;
    print_summary(&converted, &image_map, output_path);

    Ok(())
}

fn convert_docx(cli: &Cli, output_path: &PathBuf, images_base: &PathBuf) -> Result<()> {
    let docx = DocxData::open(&cli.input)?;
    let meta = docx.metadata();
    let metadata_header = metadata::format_metadata(&meta);

    // Extract images unless --no-images
    let image_map = if !cli.no_images {
        fs::create_dir_all(images_base)?;
        image::extract_images(&docx, images_base)?
    } else {
        ImageMap::new()
    };

    // DOCX chapters already have image paths set during conversion
    let chapters = docx.chapters()?;

    let converted = build_converted_chapters(&chapters)?;
    write_output(cli, output_path, &metadata_header, &converted)?;
    print_summary(&converted, &image_map, output_path);

    Ok(())
}

fn build_converted_chapters(chapters: &[Chapter]) -> Result<Vec<ConvertedChapter>> {
    let mut converted = Vec::new();

    for (i, chapter) in chapters.iter().enumerate() {
        let title = chapter
            .title
            .clone()
            .or_else(|| extract_title_from_markdown(&chapter.content))
            .unwrap_or_else(|| format!("Chapter {}", i + 1));

        let filename = format!("chapter-{:02}.md", i + 1);

        converted.push(ConvertedChapter {
            title,
            filename,
            content: chapter.content.clone(),
        });
    }

    Ok(converted)
}

fn write_output(
    cli: &Cli,
    output_path: &PathBuf,
    metadata_header: &str,
    converted: &[ConvertedChapter],
) -> Result<()> {
    if cli.single {
        write_single_file(output_path, metadata_header, converted)?;
    } else {
        write_folder(output_path, metadata_header, converted)?;
    }
    Ok(())
}

fn print_summary(converted: &[ConvertedChapter], image_map: &ImageMap, output_path: &PathBuf) {
    let chapter_count = converted.len();
    let image_count = image_map.len();
    eprintln!(
        "Converted {} chapter{}{} to {}",
        chapter_count,
        if chapter_count == 1 { "" } else { "s" },
        if image_count > 0 {
            format!(" and {} image{}", image_count, if image_count == 1 { "" } else { "s" })
        } else {
            String::new()
        },
        output_path.display()
    );
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
