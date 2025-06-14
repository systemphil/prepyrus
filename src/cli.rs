use clap::{Parser, ValueEnum};
use serde::{Deserialize, Serialize};

#[derive(Parser, Debug)]
#[command(
    name = "prepyrus",
    about = "Verify and process MDX files using a .bib file"
)]
pub struct Cli {
    /// Path to .bib file
    #[arg(long)]
    pub bib_file: String,

    /// Path to a target folder or single MDX file
    #[arg(long)]
    pub target_path: String,

    /// Operation mode: 'verify' or 'process'
    #[arg(long, value_enum)]
    pub mode: Mode,

    /// Comma-separated paths to ignore
    #[arg(long, value_delimiter = ',')]
    pub ignore_paths: Option<Vec<String>>,

    /// Optional path to write an index file (only applies when mode = process)
    #[arg(long)]
    pub generate_index_file: Option<String>,
}

#[derive(ValueEnum, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Mode {
    Verify,
    Process,
}
