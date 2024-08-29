use prepyrus::{Prepyrus, VerifiedConfig};

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let VerifiedConfig {
        bib_file,
        target_path,
        mode,
    } = Prepyrus::verify_config(&args);

    let all_entries = Prepyrus::get_all_bib_entries(&bib_file).unwrap();
    let mdx_paths = Prepyrus::get_mdx_paths(&target_path).unwrap();

    // Phase 1: Verify MDX files
    let articles_file_data = Prepyrus::verify(mdx_paths, &all_entries).unwrap();

    // Phase 2: Process MDX files (requires mode to be set to "process")
    if mode == "process" {
        Prepyrus::process(articles_file_data);
    }

    println!("===Prepyrus completed successfully!");
}
