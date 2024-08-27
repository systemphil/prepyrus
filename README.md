# Prepyrus

[![Build status](https://github.com/systemphil/prepyrus/workflows/Continuous%20integration/badge.svg)](https://github.com/systemphil/prepyrus/actions)
[![Current crates.io release](https://img.shields.io/crates/v/prepyrus)](https://crates.io/crates/prepyrus)

Prepyrus is a tool for verifying and processing MDX files
that contain citations in Chicago author-date style and certain metadata.

## Usage

Add the crate to your `Cargo.toml` and use it as shown below:

```toml
[dependencies]
prepyrus = "0.1"
```

Main API interface is the `run_prepyrus` function. Example usage:

```rust
use prepyrus::run_prepyrus;
fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 4 {
        eprintln!(
            "Expected more args. Usage: prepyrus <bibliography.bib> <target_dir_or_file> <mode>"
        );
        std::process::exit(1);
    }
    if let Err(e) = run_prepyrus(&args[1], &args[2], &args[3]) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
    println!("===Prepyrus completed successfully!");
}
```

The function takes three arguments: `<bibliography.bib> <target_dir_or_file> <mode>`

- a bibliography file (.bib),
- a target directory or .mdx file,
- and a mode (either `verify` or `process`).

`verify` mode only verifies the citations in the MDX files against the bibliography.

**⚠️ NOTE: `process` mode modifies the MDX files.**  
`process` mode _additionally_ processes the MDX files by injecting bibliography and other HTML details into the MDX files.

## Description

The tool is designed to work with MDX files that contain citations in Chicago author-date style. Examples:

```markdown
"...nowhere on heaven or on earth is there anything which does not contain both being and nothing in itself" (Hegel 2010, 61).
```

The tool parses and verifies the citations in the MDX files against a
bibliography file in BibTeX format (using Biblatex).
If the citations are valid, the tool processes the MDX files
by adding a bibliography section at the end of the file.
It also adds author, editor, and contributor from the MDX file metadata if available.
Finally, it also adds a notes heading at the end if footnotes are present in the file.

The tool has two modes: `verify` and `process`.

In `verify` mode, the tool only verifies the citations in the MDX files
and matches them against the bibliography.  
In `process` mode, the tool _additionally_ processes the MDX files by injecting bibliography
and other details into the MDX files.

## Limitations

The tool currently only supports citations in Chicago author-date style.

Only book entries are currently supported (plans to support more types in the future).

Only the following metadata fields are supported:

- author
- editor
- contributor

## License

Apache-2.0
