mod cli;
mod converter;
mod epub_reader;
mod image;
mod markdown;
mod metadata;

use anyhow::Result;
use clap::Parser;

fn main() -> Result<()> {
    let cli = cli::Cli::parse();
    converter::convert(&cli)
}
