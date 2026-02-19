# epub2md

EPUB and DOCX to Markdown converter CLI tool written in Rust.

## Architecture

```
src/
  main.rs            # Entry point, CLI parsing
  cli.rs             # Clap derive structs for CLI arguments
  reader.rs          # BookReader trait + shared types (Chapter, ImageResource, Metadata)
  converter.rs       # Orchestrates the conversion pipeline, format dispatch
  epub_reader.rs     # Wraps rbook crate, implements BookReader for EPUB
  docx_reader.rs     # Wraps docx-rust crate, implements BookReader for DOCX
  docx_markdown.rs   # OOXML element tree → Markdown conversion
  markdown.rs        # html2md conversion + shared post-processing cleanup
  image.rs           # Image extraction and path rewriting
  metadata.rs        # Metadata formatting from shared Metadata struct
```

### BookReader Trait

The `BookReader` trait in `reader.rs` provides a format-agnostic interface:
- `chapters()` → `Vec<Chapter>` (markdown content)
- `images()` → `Vec<ImageResource>` (binary image data)
- `metadata()` → `Metadata` (title, authors, etc.)

Both `EpubData` and `DocxData` implement this trait.

### EPUB Pipeline

1. Open EPUB via `rbook` with lenient parsing (`strict(false)`)
2. Extract metadata (title, author, publisher, language, description)
3. Extract images to `images/` dir, build original-path-to-new-path mapping
4. Convert each chapter's HTML to Markdown via `html2md::parse_html()`
5. Post-process: rewrite image paths, collapse blank lines, trim whitespace
6. Write output in folder mode (per-chapter .md files + README) or single-file mode

### DOCX Pipeline

1. Open DOCX via `docx-rust` (`DocxFile::from_file` → `.parse()`)
2. Extract metadata from Core/App XML properties
3. Extract images from `docx.media` HashMap
4. Walk OOXML tree (`Body > Paragraph/Table`) emitting markdown:
   - Headings via paragraph style IDs (Heading1-6, Title, Subtitle)
   - Lists via NumberingProperty (bullet/decimal format resolution)
   - Inline formatting: bold, italic, strikethrough
   - Tables with header row detection
   - Hyperlinks (internal anchors + external via relationship IDs)
   - Images via Drawing/Inline/Anchor → Blip embed → relationship resolution
5. Post-process: collapse blank lines, trim whitespace
6. Treat entire document as one chapter for output

### Key Dependencies

| Crate | License | Purpose |
|-------|---------|---------|
| `rbook` | Apache-2.0 | EPUB 2/3 parsing |
| `html2md` | MIT | HTML to Markdown conversion |
| `docx-rust` | MIT | DOCX (OOXML) parsing |
| `clap` | MIT/Apache-2.0 | CLI argument parsing |
| `anyhow` | MIT/Apache-2.0 | Error handling |

## Build & Run

```bash
cargo build
cargo run -- path/to/book.epub           # folder mode (default)
cargo run -- path/to/book.epub --single  # single file mode
cargo run -- path/to/book.epub --no-images
cargo run -- path/to/document.docx --single
```

## Conventions

- Use `anyhow::Context` for all error propagation with descriptive messages
- Keep modules focused on single responsibility
- Format-specific parsing stays in respective reader modules
- The `converter.rs` module orchestrates the pipeline and dispatches by file extension
- New formats implement `BookReader` trait in their own module
- `rbook::prelude::*` is imported in `epub_reader.rs` to bring all required traits in scope
- Image path rewriting handles various internal path formats
- The `docx-rust` `Core` and `App` types are enums with namespace variants — match both

## Testing

Currently tested manually. Test both formats and output modes:

```bash
# EPUB
cargo run -- test.epub -o /tmp/test-folder
cargo run -- test.epub --single -o /tmp/test-single.md
cargo run -- test.epub --no-images -o /tmp/test-noimages

# DOCX
cargo run -- test.docx -o /tmp/test-docx-folder
cargo run -- test.docx --single -o /tmp/test-docx.md
cargo run -- test.docx --no-images -o /tmp/test-docx-noimages
```

Verify: chapter count, image extraction, metadata in README/header, markdown quality, heading levels, lists, tables, links, formatting.
