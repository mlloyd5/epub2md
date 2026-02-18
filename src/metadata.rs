use crate::epub_reader::EpubData;

pub fn format_metadata(epub: &EpubData) -> String {
    let mut lines = Vec::new();

    if let Some(title) = epub.title() {
        lines.push(format!("# {}", title));
        lines.push(String::new());
    }

    let authors = epub.authors();
    if !authors.is_empty() {
        lines.push(format!("**Author:** {}", authors.join(", ")));
    }

    if let Some(publisher) = epub.publisher() {
        lines.push(format!("**Publisher:** {}", publisher));
    }

    if let Some(language) = epub.language() {
        lines.push(format!("**Language:** {}", language));
    }

    if let Some(description) = epub.description() {
        lines.push(String::new());
        lines.push(format!("> {}", description));
    }

    if !lines.is_empty() {
        lines.push(String::new());
        lines.push("---".to_string());
        lines.push(String::new());
    }

    let result = lines.join("\n");
    // Ensure the metadata block ends with a trailing newline
    if result.is_empty() {
        result
    } else {
        result + "\n"
    }
}
