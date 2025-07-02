# Prepyrus

[![Build status](https://github.com/systemphil/prepyrus/workflows/Continuous%20integration/badge.svg)](https://github.com/systemphil/prepyrus/actions)
[![Current crates.io release](https://img.shields.io/crates/v/prepyrus)](https://crates.io/crates/prepyrus)

Prepyrus is a tool for verifying and processing MDX files
that contain citations in Chicago author-date style and certain metadata.

⚠️ This tool is still in early development and API may frequently change.

## Usage

Add the crate to your `Cargo.toml` and use it as shown below:

```toml
[dependencies]
prepyrus = "<latest_version>"
```

Main API interface is the `Prepyrus` impl. Example usage:

```rust
use prepyrus::{
    cli::{Cli, Mode},
    Prepyrus
};

fn main() {
    let _ = run().unwrap_or_else(|e| {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    });

    println!("Prepyrus completed successfully!");
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    // Example Command Line Inputs
    let cli = Cli {
        bib_file: "tests/mocks/test.bib".to_string(),
        target_path: "tests/mocks/data-isolated".to_string(),
        mode: Mode::Verify,
        ignore_paths: Some(vec!["tests/mocks/data/development.mdx".into()]),
        generate_index_to_file: None,
        index_link_prefix_rewrite: None,
    };
    // Normally one would use let cli = Prepyrus::parse_cli();

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
```

`verify` mode only verifies the citations in the MDX files against the bibliography.

**⚠️ NOTE: This mode modifies the MDX files.**

`process` mode _additionally_ processes the MDX files by injecting bibliography and other details into the MDX files.

## Description

The tool is designed to work with MDX files that contain citations in Chicago author-date style or by BibTex key. Examples:

> "...nowhere on heaven or on earth is there anything which does not contain both being and nothing in itself" (Hegel 2010, 61).

> "The equilibrium in which coming-to-be and ceasing-to-be are poised is in the first place becoming itself" (@hegel2010logic, 81).

> "Existence proceeds from becoming" (see Hegel 2010, 61).

The tool parses and verifies the citations in the MDX files against a
bibliography file in BibTeX format (using Biblatex).
If the citations are valid, the tool processes the MDX files
by adding a bibliography section at the end of the file.
It also adds author, editor, and contributor from the MDX file metadata if available.
Finally, it also adds a notes heading at the end if footnotes are present in the file.

If BibTex keys are used, these will be replaced by disambiguated citations during `process` mode.

## Additional Features

**Alphabetical Index Generation**

When running in process mode with the `--generate-index-file <TARET_FILE>` option, Prepyrus now:

- Extracts all `indexTitles` from .mdx files.
- Sorts them alphabetically by title.
- Groups them under ## headings by first letter (e.g., ## A, ## B, etc).
- Writes a neatly structured index to the specified .mdx file.

You can rewrite parts of generated index links using:

```
--link-prefix-rewrite "/content=/articles"
```

**Handling Ambiguities**

Version `0.4` introduces citation ambiguity handling. When an author has multiple
works in the same year, such as (Hegel 1991) which might refer to the Miller
translation of the Science of Logic or the Encyclopaedia Logic, the program will
return an error with disambiguation suggestions by key. To solve ambiguous citations,
one must make use of BibTex keys prefixed with @ in the citation, e.g. `(@hegel1991logic)`.

During `process` mode, keys will be converted to disambiguated citations in Chicago author-date style.

## Limitations

The tool currently only supports citations in Chicago author-date style.
Only book entries are currently supported (plans to support more types in the future).
Only the following metadata fields are supported:

- author
- editor
- contributor

## Examples

To see a working implementation of prepyrus, please visit the [sPhil repo](https://github.com/systemphil/sphil).

## Acknowledgements

Thanks to Typst's [biblatex](https://github.com/typst/biblatex) package for providing an awesome library for parsing BibTex files, the people behind serde and regex Rust crates and the Rust community!

## License

Apache-2.0
