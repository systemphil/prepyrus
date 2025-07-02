use biblatex::{Bibliography, Chunk, Date, DateValue, Entry, PermissiveType, Spanned};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, create_dir_all, File},
    io::{self, Write},
    ops::Range,
    path::Path,
};

use crate::cli::{Cli, Mode};

/// Utility functions for prepyrus for working with BibTeX files.
pub struct BiblatexUtils;

/// Utility functions for prepyrus working with files and directories.
pub struct Utils;

#[derive(Debug)]
pub enum BibliographyError {
    IoError(std::io::Error),
    ParseError(biblatex::ParseError),
}

impl BiblatexUtils {
    /// Retrieve bibliography entries from a BibTeX file.
    pub fn retrieve_bibliography_entries(
        bibliography_path: &str,
    ) -> Result<Vec<Entry>, BibliographyError> {
        let bibliography_path =
            fs::read_to_string(bibliography_path).map_err(BibliographyError::IoError)?;
        let bibliography =
            Bibliography::parse(&bibliography_path).map_err(BibliographyError::ParseError)?;
        Ok(bibliography.into_vec())
    }

    /// Extract the year from a date that is inside of a permissive type.
    pub fn extract_year_from_date(
        date: &PermissiveType<Date>,
        reference: String,
    ) -> Result<i32, String> {
        match date {
            PermissiveType::Typed(date) => match date.value {
                DateValue::At(datetime) => Ok(datetime.year),
                DateValue::After(datetime) => Ok(datetime.year),
                DateValue::Before(datetime) => Ok(datetime.year),
                DateValue::Between(start, _end) => Ok(start.year), // Or use end.year
            },
            _ => return Err(format!("Unable to retrieve year for: {}", reference)),
        }
    }

    /// Extract volume from a permissive type.
    pub fn extract_volume(volume: &PermissiveType<i64>) -> i64 {
        match volume {
            PermissiveType::Typed(volume) => *volume,
            _ => 0,
        }
    }

    /// Extract pages from a permissive type.
    pub fn extract_pages(pages: &PermissiveType<Vec<Range<u32>>>) -> String {
        match pages {
            PermissiveType::Typed(pages) => {
                let mut pages_str = String::new();
                for page in pages {
                    pages_str.push_str(&format!("{}–{}", page.start, page.end));
                }
                pages_str
            }
            _ => String::new(),
        }
    }

    /// Use this to extract from a `Spanned<Chunk>` vector
    ///
    /// ```rust
    /// use biblatex::{Chunk, Entry, EntryType, Spanned};
    /// use prepyrus::utils::BiblatexUtils;
    ///
    /// // Mocking a Spanned<Chunk> vector
    /// let address_spanned: &[Spanned<Chunk>] = &[
    ///     Spanned {
    ///         v: Chunk::Normal("123 Fake Street".into()),
    ///         span: Default::default(),
    ///     },
    ///     Spanned {
    ///         v: Chunk::Normal("Springfield".into()),
    ///         span: Default::default(),
    ///     },
    /// ];
    ///
    /// let address: String = BiblatexUtils::extract_spanned_chunk(&address_spanned);
    /// assert_eq!(address, "123 Fake StreetSpringfield");
    /// ```
    pub fn extract_spanned_chunk(spanned_chunk: &[Spanned<Chunk>]) -> String {
        spanned_chunk
            .iter()
            .filter_map(|spanned_chunk| match spanned_chunk.v {
                Chunk::Normal(ref s) => Some(s.clone()),
                _ => None,
            })
            .collect()
    }

    /// Extract the publisher from a `Spanned<Chunk>` vector.
    pub fn extract_publisher(publisher_data: &Vec<Vec<Spanned<Chunk>>>) -> String {
        publisher_data
            .iter()
            .flat_map(|inner_vec| {
                inner_vec
                    .iter()
                    .filter_map(|spanned_chunk| match spanned_chunk.v {
                        Chunk::Normal(ref s) => Some(s.clone()),
                        _ => None,
                    })
            })
            .collect()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub bib_file: String,
    pub target_path: String,
    pub mode: Mode,
    pub settings: Settings,
    pub generate_index_file: Option<String>,
    pub index_link_prefix_rewrite: Option<(String, String)>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Settings {
    #[serde(default)]
    pub ignore_paths: Vec<String>,
}

pub enum LoadOrCreateSettingsTestMode {
    Test,
}

impl Utils {
    /// Load or create settings file.
    /// If the file does not exist, it will be created with default settings.
    fn load_or_create_settings(
        settings_path: &str,
        test_mode: Option<LoadOrCreateSettingsTestMode>,
    ) -> Result<Settings, Box<dyn std::error::Error>> {
        if let Some(LoadOrCreateSettingsTestMode::Test) = test_mode {
            return Ok(Settings {
                ignore_paths: vec!["tests/mocks/data/development.mdx".to_string()],
            });
        }
        if !std::path::Path::new(settings_path).exists() {
            create_dir_all(std::path::Path::new(settings_path).parent().unwrap())?;

            let default_settings = Settings {
                ignore_paths: Vec::new(),
            };
            let config_json = serde_json::to_string_pretty(&default_settings)?;

            let mut file = File::create(settings_path)?;
            file.write_all(config_json.as_bytes())?;
        }

        let file = File::open(settings_path)?;
        let settings: Settings = serde_json::from_reader(file)?;

        Ok(settings)
    }

    /// Extract paths of MDX files from a directory and its subdirectories.
    /// Optionally, provide a list of paths to ignore.
    pub fn extract_paths(path: &str, ignore_paths: Option<Vec<String>>) -> io::Result<Vec<String>> {
        let exceptions = ignore_paths.unwrap_or_else(|| Vec::new());
        let mdx_paths_raw = Self::extract_mdx_paths(path).unwrap();
        let mdx_paths = Self::filter_mdx_paths_for_exceptions(mdx_paths_raw, exceptions);

        Ok(mdx_paths)
    }

    /// Build configuration from arguments to be used internally.
    pub fn build_config(
        cli: Cli,
        test_mode: Option<LoadOrCreateSettingsTestMode>,
    ) -> Result<Config, &'static str> {
        // Validation (optional; `clap` can also do some of this at parse time)
        if !cli.bib_file.ends_with(".bib") {
            return Err("Invalid file format. Please provide a file with .bib extension.");
        }

        if !Path::new(&cli.target_path).is_dir() && !cli.target_path.ends_with(".mdx") {
            return Err("Invalid target. Please provide a directory or a single MDX file.");
        }

        if cli.mode != Mode::Verify && cli.mode != Mode::Process {
            return Err("Invalid mode. Please provide either 'verify' or 'process'.");
        }

        if cli.generate_index_to_file.is_some() && cli.mode != Mode::Process {
            return Err("--generate-index-file can only be used when mode is 'process'");
        }

        let settings = if let Some(paths) = cli.ignore_paths {
            Settings {
                ignore_paths: paths,
            }
        } else {
            Self::load_or_create_settings("prepyrus_settings.json", test_mode).unwrap()
        };

        Ok(Config {
            bib_file: cli.bib_file,
            target_path: cli.target_path,
            mode: cli.mode,
            settings,
            generate_index_file: cli.generate_index_to_file,
            index_link_prefix_rewrite: cli.index_link_prefix_rewrite,
        })
    }

    /// Excavates all MDX files in a directory and its subdirectories
    /// and returns a vector of paths to the MDX files.
    /// The function skips the "contributing" folder.
    fn extract_mdx_paths(path: &str) -> io::Result<Vec<String>> {
        let mut mdx_paths = Vec::new();

        if !Path::new(path).is_dir() && path.ends_with(".mdx") {
            mdx_paths.push(path.to_string());
            return Ok(mdx_paths);
        }

        let entries = fs::read_dir(path)?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                let sub_paths = Self::extract_mdx_paths(path.to_str().unwrap())?;
                mdx_paths.extend(sub_paths);
            } else if path.is_file() && path.extension() == Some(std::ffi::OsStr::new("mdx")) {
                mdx_paths.push(path.to_str().unwrap().to_string());
            }
        }
        if mdx_paths.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "No MDX files found in the directory",
            ));
        }
        Ok(mdx_paths)
    }

    /// Filter MDX paths for exceptions.
    fn filter_mdx_paths_for_exceptions(
        mdx_paths: Vec<String>,
        exceptions: Vec<String>,
    ) -> Vec<String> {
        let mut filtered_paths = mdx_paths.clone();
        if exceptions.is_empty() {
            return filtered_paths;
        }
        filtered_paths.retain(|path| !exceptions.iter().any(|exception| path.contains(exception)));
        filtered_paths
    }
}

#[cfg(test)]
mod tests_utils {
    use super::*;

    #[test]
    fn load_or_create_settings_with_test_mode() {
        let settings = Utils::load_or_create_settings(
            "test_prepyrus_settings.json",
            Some(LoadOrCreateSettingsTestMode::Test),
        )
        .expect("Failed to load or create settings");

        assert_eq!(
            settings.ignore_paths,
            vec!["tests/mocks/data/development.mdx"]
        );
    }

    #[test]
    fn load_or_create_settings_with_dummy_data() {
        let test_settings_path = "test_prepyrus_settings.json";

        // Setup: make sure test starts with no existing file
        if std::path::Path::new(test_settings_path).exists() {
            fs::remove_file(test_settings_path)
                .expect("Failed to remove existing test settings file");
        }

        // 1. Create file with default settings
        let _ = Utils::load_or_create_settings(test_settings_path, None)
            .expect("Failed to create settings");
        assert!(std::path::Path::new(test_settings_path).exists());

        // 2. Write to file with test settings
        let mut file = fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(test_settings_path)
            .expect("Failed to open the settings file for writing");
        let modified_settings = Settings {
            ignore_paths: vec![
                "tests/mocks/data/engels.mdx".to_string(),
                "tests/mocks/data/marx.mdx".to_string(),
            ],
        };
        let config_json = serde_json::to_string_pretty(&modified_settings)
            .expect("Failed to serialize modified settings");
        file.write_all(config_json.as_bytes())
            .expect("Failed to write to the settings file");

        // 3. Read and verify test settings file
        let reloaded_settings = Utils::load_or_create_settings(test_settings_path, None)
            .expect("Failed to reload settings");
        assert_eq!(
            reloaded_settings.ignore_paths,
            vec!["tests/mocks/data/engels.mdx", "tests/mocks/data/marx.mdx"]
        );

        // Cleanup: remove test settings file
        fs::remove_file(test_settings_path).expect("Failed to remove the test settings file");
        assert!(!std::path::Path::new(test_settings_path).exists());
    }
}
