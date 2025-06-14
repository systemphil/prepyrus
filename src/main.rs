use prepyrus::{cli::Mode, Prepyrus};

fn main() {
    let _ = run().unwrap_or_else(|e| {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    });

    println!("✅ Prepyrus completed successfully!");
}

/// Run all the methods of prepyrus
fn run() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Prepyrus::parse_cli();
    let config = Prepyrus::build_config(cli, None)?;
    let all_entries = Prepyrus::get_all_bib_entries(&config.bib_file).unwrap();
    let mdx_paths =
        Prepyrus::get_mdx_paths(&config.target_path, Some(config.settings.ignore_paths))?;

    // Phase 1: Verify MDX files
    let articles_file_data = Prepyrus::verify(mdx_paths, &all_entries)?;

    // Phase 2: Process MDX files (requires mode to be set to "process")
    if config.mode == Mode::Process {
        Prepyrus::process(articles_file_data);
    }

    Ok(())
}
