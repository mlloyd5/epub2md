mod cli;
mod converter;
mod docx_markdown;
mod docx_reader;
mod epub_reader;
mod image;
mod markdown;
mod metadata;
mod reader;

use anyhow::Result;
use clap::Parser;

fn main() -> Result<()> {
    let cli = cli::Cli::parse();
    converter::convert(&cli)
}
