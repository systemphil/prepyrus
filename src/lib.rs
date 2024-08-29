/*!
Prepyrus is a tool for verifying and processing MDX files
that contain citations in Chicago author-date style and certain metadata.

## Usage

Add the crate to your `Cargo.toml` and use it as shown below:

```toml
[dependencies]
prepyrus = "0.1"
```

Main API interface is the `run_prepyrus` function. Example usage:

```rust,ignore
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

**⚠️ NOTE: This mode modifies the MDX files.**

`process` mode _additionally_ processes the MDX files by injecting bibliography and other details into the MDX files.

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
*/

pub mod inserter;
pub mod utils;
pub mod validator;

use std::io::Error;

pub use crate::utils::VerifiedConfig;
use biblatex::Entry;
use utils::{BiblatexUtils, CoreUtils};
use validator::ArticleFileData;
pub struct Prepyrus {}

impl Prepyrus {
    pub fn verify_config(args: &Vec<String>) -> VerifiedConfig {
        CoreUtils::verify_config(args)
    }

    pub fn get_all_bib_entries(
        bib_file: &str,
    ) -> Result<Vec<biblatex::Entry>, Box<dyn std::error::Error>> {
        Ok(BiblatexUtils::retrieve_bibliography_entries(bib_file)?)
    }

    pub fn get_mdx_paths(target_path: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        Ok(CoreUtils::extract_paths(target_path)?)
    }

    pub fn verify(
        mdx_paths: Vec<String>,
        all_entries: &Vec<Entry>,
    ) -> Result<Vec<ArticleFileData>, Error> {
        validator::verify_mdx_files(mdx_paths, &all_entries)
    }

    pub fn process(all_articles: Vec<ArticleFileData>) {
        inserter::process_mdx_files(all_articles)
    }
}
