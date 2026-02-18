use clap::Parser;
use std::path::PathBuf;

/// Convert EPUB ebooks to clean Markdown
#[derive(Parser, Debug)]
#[command(name = "epub2md", version, about)]
pub struct Cli {
    /// Path to the input EPUB file
    pub input: PathBuf,

    /// Output path (directory for folder mode, file for single-file mode).
    /// Defaults to a directory or file named after the EPUB in the current directory.
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Output as a single combined Markdown file instead of a directory of chapter files
    #[arg(short, long, default_value_t = false)]
    pub single: bool,

    /// Do not extract images (only convert text content)
    #[arg(long, default_value_t = false)]
    pub no_images: bool,
}
