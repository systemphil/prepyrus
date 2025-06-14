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

    /// Optional path to generate to an index file (only applies when mode = process)
    #[arg(long)]
    pub generate_index_to_file: Option<String>,

    /// Optional rewrite of generated index link (relevant only when generating index)
    #[arg(long, value_parser = parse_prefix_rewrite)]
    pub index_link_prefix_rewrite: Option<(String, String)>,
}

#[derive(ValueEnum, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Mode {
    Verify,
    Process,
}

/// Parses a string of the form "from:to" into a tuple (from, to)
fn parse_prefix_rewrite(s: &str) -> Result<(String, String), String> {
    let parts: Vec<&str> = s.splitn(2, ':').collect();
    if parts.len() != 2 {
        return Err("Expected format: <from>:<to>".to_string());
    }
    Ok((parts[0].to_string(), parts[1].to_string()))
}
