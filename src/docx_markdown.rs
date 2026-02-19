use crate::image::ImageMap;
use docx_rust::document::{
    BodyContent, ParagraphContent, RunContent, TableCellContent, TableRowContent,
};
use docx_rust::formatting::CharacterProperty;
use docx_rust::Docx;

/// Convert a parsed DOCX document body to markdown
pub fn docx_to_markdown(docx: &Docx, image_map: &ImageMap) -> String {
    let mut ctx = ConvertContext {
        docx,
        image_map,
        output: String::new(),
        list_counters: std::collections::HashMap::new(),
    };

    for content in &docx.document.body.content {
        ctx.convert_body_content(content);
    }

    ctx.output
}

struct ConvertContext<'a> {
    docx: &'a Docx<'a>,
    image_map: &'a ImageMap,
    output: String,
    /// Track numbering counters: (num_id, level) -> current count
    list_counters: std::collections::HashMap<(isize, isize), usize>,
}

impl<'a> ConvertContext<'a> {
    fn convert_body_content(&mut self, content: &BodyContent) {
        match content {
            BodyContent::Paragraph(para) => self.convert_paragraph(para),
            BodyContent::Table(table) => self.convert_table(table),
            BodyContent::Sdt(sdt) => {
                // SDT has content: Option<SDTContent> which has content: Vec<BodyContent>
                if let Some(ref sdt_content) = sdt.content {
                    for item in &sdt_content.content {
                        self.convert_body_content(item);
                    }
                }
            }
            _ => {}
        }
    }

    fn convert_paragraph(&mut self, para: &docx_rust::document::Paragraph) {
        let mut heading_level: Option<u8> = None;
        let mut numbering: Option<(isize, isize)> = None; // (num_id, level)

        if let Some(ref prop) = para.property {
            // Detect heading via style ID
            if let Some(ref style_id) = prop.style_id {
                let id = style_id.value.as_ref();
                heading_level = match id {
                    "Heading1" | "heading1" | "heading 1" => Some(1),
                    "Heading2" | "heading2" | "heading 2" => Some(2),
                    "Heading3" | "heading3" | "heading 3" => Some(3),
                    "Heading4" | "heading4" | "heading 4" => Some(4),
                    "Heading5" | "heading5" | "heading 5" => Some(5),
                    "Heading6" | "heading6" | "heading 6" => Some(6),
                    "Title" | "title" => Some(1),
                    "Subtitle" | "subtitle" => Some(2),
                    _ => None,
                };
            }

            // Detect list numbering — both id and level are Option<T>
            if let Some(ref num_prop) = prop.numbering {
                if let (Some(ref id), Some(ref level)) = (&num_prop.id, &num_prop.level) {
                    numbering = Some((id.value, level.value));
                }
            }
        }

        // Collect inline content (runs + hyperlinks)
        let inline_md = self.collect_inline_content(para);

        // Skip empty paragraphs
        if inline_md.trim().is_empty() && heading_level.is_none() && numbering.is_none() {
            self.output.push('\n');
            return;
        }

        // Emit heading prefix
        if let Some(level) = heading_level {
            let prefix: String = "#".repeat(level as usize);
            self.output.push_str(&prefix);
            self.output.push(' ');
            self.output.push_str(inline_md.trim());
            self.output.push_str("\n\n");
            return;
        }

        // Emit list item
        if let Some((num_id, level)) = numbering {
            let indent = "  ".repeat(level as usize);
            let bullet = self.resolve_list_bullet(num_id, level);
            self.output.push_str(&indent);
            self.output.push_str(&bullet);
            self.output.push(' ');
            self.output.push_str(inline_md.trim());
            self.output.push('\n');
            return;
        }

        // Regular paragraph
        self.output.push_str(inline_md.trim());
        self.output.push_str("\n\n");
    }

    fn collect_inline_content(&mut self, para: &docx_rust::document::Paragraph) -> String {
        let mut result = String::new();

        for pc in &para.content {
            match pc {
                ParagraphContent::Run(run) => {
                    let text = self.collect_run_text(run);
                    if !text.is_empty() {
                        let formatted = format_run_text(&text, &run.property);
                        result.push_str(&formatted);
                    }
                }
                ParagraphContent::Link(link) => {
                    let display_text = link
                        .content
                        .as_ref()
                        .map(|run| self.collect_run_text(run))
                        .unwrap_or_default();

                    let target = self.resolve_hyperlink_target(link);

                    if let Some(url) = target {
                        if !display_text.is_empty() {
                            result.push_str(&format!("[{}]({})", display_text, url));
                        } else {
                            result.push_str(&url);
                        }
                    } else {
                        result.push_str(&display_text);
                    }
                }
                _ => {}
            }
        }

        result
    }

    fn collect_run_text(&mut self, run: &docx_rust::document::Run) -> String {
        let mut text = String::new();

        for rc in &run.content {
            match rc {
                RunContent::Text(t) => text.push_str(&t.text),
                RunContent::Break(_) => text.push('\n'),
                RunContent::Tab(_) => text.push('\t'),
                RunContent::Drawing(drawing) => {
                    if let Some(md) = self.convert_drawing(drawing) {
                        text.push_str(&md);
                    }
                }
                _ => {}
            }
        }

        text
    }

    fn convert_drawing(&self, drawing: &docx_rust::document::Drawing) -> Option<String> {
        // Try inline drawing first (most common)
        if let Some(ref inline) = drawing.inline {
            if let Some(ref graphic) = inline.graphic {
                // GraphicData has `children: Vec<Picture>`, not a direct `pic` field
                if let Some(pic) = graphic.data.children.first() {
                    let embed_id = pic.fill.blip.embed.as_ref();
                    let alt = inline.doc_property.descr.as_deref().unwrap_or("");
                    return self.resolve_image(embed_id, alt);
                }
            }
        }

        // Try anchor (floating images)
        if let Some(ref anchor) = drawing.anchor {
            if let Some(ref graphic) = anchor.graphic {
                if let Some(pic) = graphic.data.children.first() {
                    let embed_id = pic.fill.blip.embed.as_ref();
                    let alt = anchor.doc_property.descr.as_deref().unwrap_or("");
                    return self.resolve_image(embed_id, alt);
                }
            }
        }

        None
    }

    fn resolve_image(&self, embed_id: &str, alt: &str) -> Option<String> {
        // Resolve relationship ID to file path
        let target = self
            .docx
            .document_rels
            .as_ref()?
            .relationships
            .iter()
            .find(|r| r.id.as_ref() == embed_id)?
            .target
            .as_ref();

        // Check if we have this image in our image map
        let image_path = if let Some(mapped) = self.image_map.get(target) {
            mapped.clone()
        } else {
            // Try with "word/" prefix since DOCX stores images as word/media/...
            let full_path = format!("word/{}", target);
            if let Some(mapped) = self.image_map.get(&full_path) {
                mapped.clone()
            } else {
                // Fallback: use the target path directly
                format!("images/{}", target.rsplit('/').next().unwrap_or(target))
            }
        };

        Some(format!("![{}]({})", alt, image_path))
    }

    fn resolve_hyperlink_target(&self, link: &docx_rust::document::Hyperlink) -> Option<String> {
        // Internal anchor link
        if let Some(ref anchor) = link.anchor {
            return Some(format!("#{}", anchor));
        }

        // External link via relationship ID
        if let Some(ref id) = link.id {
            if let Some(ref rels) = self.docx.document_rels {
                for r in &rels.relationships {
                    if r.id.as_ref() == id.as_ref() {
                        return Some(r.target.to_string());
                    }
                }
            }
        }

        None
    }

    fn resolve_list_bullet(&mut self, num_id: isize, level: isize) -> String {
        if let Some(ref numbering) = self.docx.numbering {
            // Find the Num entry for this num_id
            for num in &numbering.numberings {
                if num.num_id == Some(num_id) {
                    // Get the abstract numbering ID
                    let abstract_id = match &num.abstract_num_id {
                        Some(aid) => aid.value,
                        None => continue,
                    };

                    // Find the matching abstract numbering
                    for abstract_num in &numbering.abstract_numberings {
                        if abstract_num.abstract_num_id == abstract_id {
                            for lvl in &abstract_num.levels {
                                if lvl.i_level == Some(level) {
                                    if let Some(ref fmt) = lvl.number_format {
                                        let fmt_val = fmt.value.as_ref();
                                        return match fmt_val {
                                            "bullet" => "-".to_string(),
                                            "decimal" | "upperRoman" | "lowerRoman"
                                            | "upperLetter" | "lowerLetter" => {
                                                let counter = self
                                                    .list_counters
                                                    .entry((num_id, level))
                                                    .or_insert(0);
                                                *counter += 1;
                                                format!("{}.", counter)
                                            }
                                            "none" => "-".to_string(),
                                            _ => "-".to_string(),
                                        };
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Default to bullet if we can't resolve
        "-".to_string()
    }

    fn convert_table(&mut self, table: &docx_rust::document::Table) {
        let mut rows: Vec<Vec<String>> = Vec::new();

        for row in &table.rows {
            let mut cells: Vec<String> = Vec::new();

            for cell_content in &row.cells {
                if let TableRowContent::TableCell(cell) = cell_content {
                    let cell_text = self.collect_cell_text(cell);
                    cells.push(cell_text);
                }
            }

            if !cells.is_empty() {
                rows.push(cells);
            }
        }

        if rows.is_empty() {
            return;
        }

        // Determine column count
        let col_count = rows.iter().map(|r| r.len()).max().unwrap_or(0);

        // Emit markdown table
        for (i, row) in rows.iter().enumerate() {
            self.output.push('|');
            for j in 0..col_count {
                let cell = row.get(j).map(|s| s.as_str()).unwrap_or("");
                self.output.push(' ');
                self.output.push_str(cell);
                self.output.push_str(" |");
            }
            self.output.push('\n');

            // Add header separator after first row
            if i == 0 {
                self.output.push('|');
                for _ in 0..col_count {
                    self.output.push_str(" --- |");
                }
                self.output.push('\n');
            }
        }
        self.output.push('\n');
    }

    fn collect_cell_text(&mut self, cell: &docx_rust::document::TableCell) -> String {
        let mut parts: Vec<String> = Vec::new();

        for tc in &cell.content {
            let TableCellContent::Paragraph(para) = tc;
            let text = self.collect_inline_content(para);
            let trimmed = text.trim().to_string();
            if !trimmed.is_empty() {
                parts.push(trimmed);
            }
        }

        // Join multiple paragraphs in a cell with <br>
        parts.join("<br>")
    }
}

/// Wrap text in markdown formatting based on run properties
fn format_run_text(text: &str, props: &Option<CharacterProperty>) -> String {
    let Some(props) = props else {
        return text.to_string();
    };

    let is_bold = props
        .bold
        .as_ref()
        .map(|b| b.value != Some(false))
        .unwrap_or(false);
    let is_italic = props
        .italics
        .as_ref()
        .map(|i| i.value != Some(false))
        .unwrap_or(false);
    let is_strike = props.strike.is_some() || props.dstrike.is_some();

    // Don't wrap whitespace-only text
    if text.trim().is_empty() {
        return text.to_string();
    }

    let mut result = text.to_string();

    if is_strike {
        result = format!("~~{}~~", result);
    }
    if is_bold && is_italic {
        result = format!("***{}***", result);
    } else if is_bold {
        result = format!("**{}**", result);
    } else if is_italic {
        result = format!("*{}*", result);
    }

    result
}
