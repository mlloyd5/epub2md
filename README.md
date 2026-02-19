# epub2md

Convert EPUB and DOCX documents to clean Markdown files suitable for LLM and agentic consumption.

## Installation

Build from source:

```bash
git clone https://github.com/mlloyd5/epub2md
cd epub2md
cargo build --release
```

The binary will be at `target/release/epub2md`.

## Usage

```bash
# Convert EPUB to a directory of chapter files (default)
epub2md book.epub

# Convert EPUB to a single markdown file
epub2md book.epub --single

# Convert DOCX to markdown
epub2md document.docx --single

# Specify output location
epub2md book.epub -o ./output/

# Skip image extraction
epub2md book.epub --no-images
```

### Supported Formats

| Format | Extension | Features |
|--------|-----------|----------|
| EPUB | `.epub` | Chapters, images, metadata, EPUB 2/3 |
| DOCX | `.docx` | Headings, lists, tables, images, links, bold/italic/strikethrough, metadata |

## Output Formats

### Folder Mode (default)

Creates a directory with individual chapter files, a README with metadata and table of contents, and an `images/` subdirectory for extracted images.

```
book-name/
  README.md           # Metadata + table of contents with links
  chapter-01.md
  chapter-02.md
  ...
  images/
    cover.jpg
    figure1.png
```

### Single File Mode (`--single`)

Creates one combined Markdown file with all chapters separated by horizontal rules, plus a sibling `images/` directory.

```
book-name.md
images/
  cover.jpg
  figure1.png
```

## Dependencies

- [rbook](https://crates.io/crates/rbook) (Apache-2.0) - EPUB 2/3 parsing
- [html2md](https://crates.io/crates/html2md) (MIT) - HTML to Markdown conversion
- [docx-rust](https://crates.io/crates/docx-rust) (MIT) - DOCX (OOXML) parsing
- [clap](https://crates.io/crates/clap) - CLI argument parsing
- [anyhow](https://crates.io/crates/anyhow) - Error handling

## License

MIT
