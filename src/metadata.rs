use crate::reader::Metadata;

pub fn format_metadata(meta: &Metadata) -> String {
    let mut lines = Vec::new();

    if let Some(ref title) = meta.title {
        if !title.trim().is_empty() {
            lines.push(format!("# {}", title));
            lines.push(String::new());
        }
    }

    let non_empty_authors: Vec<_> = meta
        .authors
        .iter()
        .filter(|a| !a.trim().is_empty())
        .collect();
    if !non_empty_authors.is_empty() {
        let joined: Vec<_> = non_empty_authors.iter().map(|a| a.as_str()).collect();
        lines.push(format!("**Author:** {}", joined.join(", ")));
    }

    if let Some(ref publisher) = meta.publisher {
        if !publisher.trim().is_empty() {
            lines.push(format!("**Publisher:** {}", publisher));
        }
    }

    if let Some(ref language) = meta.language {
        if !language.trim().is_empty() {
            lines.push(format!("**Language:** {}", language));
        }
    }

    if let Some(ref description) = meta.description {
        if !description.trim().is_empty() {
            lines.push(String::new());
            lines.push(format!("> {}", description));
        }
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
