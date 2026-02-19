use crate::reader::BookReader;
use anyhow::Result;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Maps original image hrefs to their new relative paths in output
pub type ImageMap = HashMap<String, String>;

pub fn extract_images(reader: &dyn BookReader, output_dir: &Path) -> Result<ImageMap> {
    let images_dir = output_dir.join("images");
    let images = reader.images()?;

    if images.is_empty() {
        return Ok(ImageMap::new());
    }

    fs::create_dir_all(&images_dir)?;

    let mut image_map = ImageMap::new();

    for img in &images {
        let filename = clean_filename(&img.original_href);
        let dest = images_dir.join(&filename);

        fs::write(&dest, &img.data)?;

        image_map.insert(
            img.original_href.clone(),
            format!("images/{}", filename),
        );
    }

    Ok(image_map)
}

fn clean_filename(href: &str) -> String {
    Path::new(href)
        .file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_else(|| "image.bin".to_string())
}
