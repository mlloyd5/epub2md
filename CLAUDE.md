# epub2md

EPUB to Markdown converter CLI tool written in Rust.

## Architecture

```
src/
  main.rs            # Entry point, CLI parsing
  cli.rs             # Clap derive structs for CLI arguments
  converter.rs       # Orchestrates the conversion pipeline
  epub_reader.rs     # Wraps rbook crate, provides Chapter/Image abstractions
  markdown.rs        # html2md conversion + post-processing cleanup
  image.rs           # Image extraction and path rewriting
  metadata.rs        # Metadata extraction and formatting
```

### Conversion Pipeline

1. Open EPUB via `rbook` with lenient parsing (`strict(false)`)
2. Extract metadata (title, author, publisher, language, description)
3. Extract images to `images/` dir, build original-path-to-new-path mapping
4. Convert each chapter's HTML to Markdown via `html2md::parse_html()`
5. Post-process: rewrite image paths, collapse blank lines, trim whitespace
6. Write output in folder mode (per-chapter .md files + README) or single-file mode

### Key Dependencies

| Crate | License | Purpose |
|-------|---------|---------|
| `rbook` | Apache-2.0 | EPUB 2/3 parsing |
| `html2md` | MIT | HTML to Markdown conversion |
| `clap` | MIT/Apache-2.0 | CLI argument parsing |
| `anyhow` | MIT/Apache-2.0 | Error handling |

## Build & Run

```bash
cargo build
cargo run -- path/to/book.epub           # folder mode (default)
cargo run -- path/to/book.epub --single  # single file mode
cargo run -- path/to/book.epub --no-images
```

## Conventions

- Use `anyhow::Context` for all error propagation with descriptive messages
- Keep modules focused on single responsibility
- EPUB parsing details stay in `epub_reader.rs`
- The `converter.rs` module orchestrates the pipeline without knowing implementation details
- `rbook::prelude::*` is imported in `epub_reader.rs` to bring all required traits in scope
- Image path rewriting handles various EPUB internal path formats (absolute, relative, filename-only)

## Testing

Currently tested manually with public domain EPUBs from Project Gutenberg. Test both output modes:

```bash
cargo run -- test.epub -o /tmp/test-folder
cargo run -- test.epub --single -o /tmp/test-single.md
cargo run -- test.epub --no-images -o /tmp/test-noimages
```

Verify: chapter count, image extraction, metadata in README/header, markdown quality.
