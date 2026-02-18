use crate::image::ImageMap;

pub fn html_to_markdown(html: &str, image_map: &ImageMap) -> String {
    let mut md = html2md::parse_html(html);

    // Rewrite image paths from EPUB-internal paths to extracted paths
    for (original, replacement) in image_map {
        md = rewrite_image_path(&md, original, replacement);
    }

    clean_markdown(&md)
}

fn rewrite_image_path(md: &str, original: &str, replacement: &str) -> String {
    let mut result = md.replace(original, replacement);

    // Also try matching just the filename portion, since EPUB refs
    // can use varying relative paths (../Images/fig.png vs Images/fig.png vs fig.png)
    if let Some(filename) = std::path::Path::new(original).file_name() {
        let filename_str = filename.to_string_lossy();
        // Only replace if it's inside a markdown image/link pattern to avoid false positives
        let patterns = [
            format!("]({})", filename_str),
            format!("\"{}\"", filename_str),
        ];
        for pattern in &patterns {
            if result.contains(pattern.as_str()) {
                let new_pattern = pattern.replace(filename_str.as_ref(), replacement);
                result = result.replace(pattern.as_str(), &new_pattern);
            }
        }
    }

    result
}

fn clean_markdown(md: &str) -> String {
    let mut result = md.to_string();

    // Collapse 3+ consecutive blank lines to 2
    while result.contains("\n\n\n") {
        result = result.replace("\n\n\n", "\n\n");
    }

    // Trim trailing whitespace per line
    result = result
        .lines()
        .map(|line| line.trim_end())
        .collect::<Vec<_>>()
        .join("\n");

    // Ensure single trailing newline
    let trimmed = result.trim_end().to_string();
    if trimmed.is_empty() {
        String::new()
    } else {
        trimmed + "\n"
    }
}
