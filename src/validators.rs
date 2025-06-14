use crate::BiblatexUtils;
use biblatex::Entry;
use regex::Regex;
use serde::Deserialize;
use std::fs;
use std::io::{self, BufReader, Error, Read};

#[derive(Debug, Deserialize)]
pub struct Metadata {
    pub title: String,
    #[serde(rename = "indexTitle")]
    pub index_title: String,
    pub description: String,
    #[serde(rename = "isArticle")]
    pub is_article: bool,
    pub authors: Option<String>,
    pub editors: Option<String>,
    pub contributors: Option<String>,
}

#[derive(Debug)]
pub struct ArticleFileData {
    /// Path to the file whose contents were extracted.
    pub path: String,
    /// Metadata enclosed at the top of the file.
    pub metadata: Metadata,
    /// Contents of the file.
    pub markdown_content: String,
    /// A set of citations that exist in the source `.bib` file.
    pub matched_citations: Vec<Entry>,
    /// Original contents of the file, includes metadata.
    pub full_file_content: String,
}

/// Verifies the integrity of MDX files.
/// The function reads the MDX files, extracts metadata and markdown content,
/// verifies the citations format, and matches the citations to the bibliography.
/// The function returns a list of ArticleFileData structs containing the metadata,
/// markdown content, matched citations, and full file content.
pub fn verify_mdx_files(
    mdx_paths: Vec<String>,
    all_entries: &Vec<Entry>,
) -> Result<Vec<ArticleFileData>, Error> {
    let mut article_count = 0;
    let mut all_articles: Vec<ArticleFileData> = Vec::new();
    for mdx_path in &mdx_paths {
        let (metadata, markdown_content, full_file_content) = match read_mdx_file(&mdx_path) {
            Ok(data) => data,
            Err(err) => {
                if err.kind() == io::ErrorKind::InvalidData {
                    eprintln!("Invalid MDX data format: {}", err);
                    std::process::exit(1);
                } else {
                    eprintln!("Unexpected error reading MDX file: {}", err);
                    std::process::exit(1);
                }
            }
        };
        if !metadata.is_article {
            continue;
        }
        if !check_parentheses_balance(&markdown_content) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Unbalanced parentheses in {}", mdx_path),
            ));
        }
        let citations = extract_citations_from_markdown(&markdown_content);
        match verify_citations_format(&citations) {
            Ok(_) => {}
            Err(err) => {
                eprintln!("Error verifying citations: {} in {}", err, mdx_path);
                std::process::exit(1);
            }
        };
        let citations_set = create_citations_set(citations);
        let matched_citations = match match_citations_to_bibliography(citations_set, &all_entries) {
            Ok(data) => data,
            Err(err) => {
                eprintln!(
                    "Error matching citations to bibliography: {} in {}",
                    err, mdx_path
                );
                std::process::exit(1);
            }
        };
        all_articles.push(ArticleFileData {
            path: mdx_path.clone(),
            metadata,
            markdown_content,
            matched_citations,
            full_file_content,
        });
        article_count += 1;
    }
    println!(
        "✓ Integrity verification OK: {} files verified, including {} articles",
        mdx_paths.len(),
        article_count
    );
    Ok(all_articles)
}

/// Reads an MDX file and extracts metadata and markdown content.
/// The function returns a tuple containing the metadata, markdown content, and full file content.
/// The metadata is expected to be enclosed in `---` at the start of the file.
fn read_mdx_file(path: &str) -> io::Result<(Metadata, String, String)> {
    let file = fs::File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut content = String::new();
    reader.read_to_string(&mut content)?;

    // Extract metadata enclosed in `---` at the start of the file
    let parts: Vec<&str> = content.splitn(3, "---").collect();
    if parts.len() != 3 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Unable to extract metadata in {}", path),
        ));
    }

    let metadata_str = parts[1];
    let metadata: Metadata = match serde_yaml::from_str(metadata_str) {
        Ok(data) => data,
        Err(err) => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("{} in {}", err, path),
            ))
        }
    };
    let markdown_content = parts[2].to_string();
    let full_file_content = content.clone();

    Ok((metadata, markdown_content, full_file_content))
}

/// Checks if the parentheses in a markdown string are balanced.
/// No odd number of parentheses is allowed.
fn check_parentheses_balance(markdown: &String) -> bool {
    let mut balance = 0;

    for ch in markdown.chars() {
        if ch == '(' {
            balance += 1;
        } else if ch == ')' {
            balance -= 1;
        }

        if balance < 0 {
            return false;
        }
    }

    balance == 0
}

/// Extract citations from a markdown string
/// The citations are assumed to be Chicago author-date style
/// and in the format (Author_last_name 2021) or (Author_last_name 2021, 123)
///
/// ### Example
///
/// (Hegel 2021) or (Hegel 2021, 123)
fn extract_citations_from_markdown(markdown: &String) -> Vec<String> {
    //      Regex explanation
    //
    //      \(      Match an opening parenthesis
    //     (see\s)? Optionally match the word "see" followed by a whitespace
    //      ([A-Z]  Match a capital letter
    //      [^()]*? Match any character except opening and closing parenthesis
    //      \d+     Match one or more digits
    //      (?:     Start a non-capturing group
    //      ,       Match a comma
    //      [^)]*   Match any character except closing parenthesis
    //      )?      End the non-capturing group and make it optional
    //      \)      Match a closing parenthesis
    //
    // The regex will match citations in the format (Author_last_name 2021) or (Author_last_name 2021, 123)
    //
    let citation_regex = Regex::new(r"\((see\s)?([A-Z][^()]*?\d+(?:,[^)]*)?)\)").unwrap();
    let mut citations = Vec::new();

    for line in markdown.lines() {
        for captures in citation_regex.captures_iter(line) {
            match captures.len() {
                2 => {
                    let citation = captures.get(1).unwrap().as_str().trim();
                    citations.push(citation.to_string());
                }
                3 => {
                    let citation = captures.get(2).unwrap().as_str().trim();
                    citations.push(citation.to_string());
                }
                _ => {} // Ignore unexpected capture group lengths
            }
        }
    }
    citations
}

/// Verifies the format of the citations extracted from the markdown.
/// The citations are expected to be in the format (Author_last_name 2021)
/// or (Author_last_name 2021, 123)
fn verify_citations_format(citations: &Vec<String>) -> Result<(), io::Error> {
    for citation in citations {
        let citation_split = citation.splitn(2, ',').collect::<Vec<&str>>();
        let first_part = citation_split[0].trim();
        let has_year = first_part.split_whitespace().any(|word| {
            if let Ok(num) = word.parse::<u32>() {
                num >= 1000 && num <= 9999
            } else {
                false
            }
        });
        if !has_year {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Citation is malformed or is missing year: ({})", citation),
            ));
        }
    }
    Ok(())
}

/// Creates a set of unique citations from a list of citations.
fn create_citations_set(citations: Vec<String>) -> Vec<String> {
    let mut citations_set = Vec::new();
    for citation in citations {
        let prepared_citation = citation
            .splitn(2, ',')
            .next()
            .unwrap_or(&citation)
            .to_string();
        if !citations_set.contains(&prepared_citation) {
            citations_set.push(prepared_citation);
        }
    }
    citations_set
}

/// Matches citations to the inputted bibliography
/// the matched list is returned with full bibliographical details.
/// If any citation is not found in the bibliography, an error is returned.
fn match_citations_to_bibliography(
    citations: Vec<String>,
    bibliography: &Vec<Entry>,
) -> Result<Vec<Entry>, io::Error> {
    let mut unmatched_citations = citations.clone();
    let mut matched_citations = Vec::new();

    for citation in citations {
        for entry in bibliography {
            let author = entry.author().unwrap();
            let author_last_name = author[0].name.clone();

            let date: biblatex::PermissiveType<biblatex::Date> = entry.date().unwrap();
            let year = BiblatexUtils::extract_year_from_date(&date, citation.clone()).unwrap();

            let author_year = format!("{} {:?}", author_last_name, year);

            if citation == author_year {
                unmatched_citations.retain(|x| x != &citation);
                matched_citations.push(entry.clone());
            }
        }
    }

    if unmatched_citations.len() > 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "Citations not found in the library: ({:?})",
                unmatched_citations
            ),
        ));
    }

    Ok(matched_citations)
}

#[cfg(test)]
mod tests_balanced_parentheses {
    use super::*;

    #[test]
    fn balanced_parentheses() {
        let markdown = String::from("This is a balanced citation (Spinoza 2021).");
        assert!(check_parentheses_balance(&markdown));
    }
    #[test]
    fn unbalanced_parentheses_more_open() {
        let markdown = String::from("This is an unbalanced citation (Spinoza 2021.");
        assert!(!check_parentheses_balance(&markdown));
    }
    #[test]
    fn unbalanced_parentheses_more_close() {
        let markdown = String::from("This is an unbalanced citation Spinoza 2021).");
        assert!(!check_parentheses_balance(&markdown));
    }
}

#[cfg(test)]
mod tests_citation_extraction {
    use super::*;

    #[test]
    fn single_citation() {
        let markdown = String::from("This is a citation (Hegel 2021) in the text.");
        let citations = extract_citations_from_markdown(&markdown);
        assert_eq!(citations, vec!["Hegel 2021"]);
    }
    #[test]
    fn single_citation_prefixed_see() {
        let markdown = String::from("This is a citation (see Hegel 2021) in the text.");
        let citations = extract_citations_from_markdown(&markdown);
        assert_eq!(citations, vec!["Hegel 2021"]);
    }
    #[test]
    fn multiple_citations() {
        let markdown =
            String::from("This is a citation (Spinoza 2021) and another one (Kant 2020, 123).");
        let citations = extract_citations_from_markdown(&markdown);
        assert_eq!(citations, vec!["Spinoza 2021", "Kant 2020, 123"]);
    }
    #[test]
    fn multiple_citations_prefixed_see() {
        let markdown = String::from(
            "This is a citation (see Spinoza 2021) and another one (see Kant 2020, 123).",
        );
        let citations = extract_citations_from_markdown(&markdown);
        assert_eq!(citations, vec!["Spinoza 2021", "Kant 2020, 123"]);
    }
    #[test]
    fn no_citation() {
        let markdown = String::from("This text has no citations.");
        let citations = extract_citations_from_markdown(&markdown);
        assert_eq!(citations, Vec::<String>::new());
    }
    #[test]
    fn citation_with_additional_text() {
        let markdown = String::from("This citation (Plato 2019) has additional text.");
        let citations = extract_citations_from_markdown(&markdown);
        assert_eq!(citations, vec!["Plato 2019"]);
    }
    #[test]
    fn multiple_lines() {
        let markdown = String::from(
            "First citation (Aristotle 2020).\n\
            Second citation on a new line (Hume 2018).\n\
            No citation here.",
        );
        let citations = extract_citations_from_markdown(&markdown);
        assert_eq!(citations, vec!["Aristotle 2020", "Hume 2018"]);
    }
    #[test]
    fn incomplete_citation_opening_parenthesis_only() {
        let markdown = String::from("This is an incomplete citation (Spinoza 2021.");
        let valid_citations = extract_citations_from_markdown(&markdown);
        assert!(valid_citations.is_empty());
    }
    #[test]
    fn incomplete_citation_closing_parenthesis_only() {
        let markdown = String::from("This is an incomplete citation Descartes 2021).");
        let valid_citations = extract_citations_from_markdown(&markdown);
        assert!(valid_citations.is_empty());
    }
    #[test]
    fn mixed_valid_and_invalid_citations() {
        let markdown =
            String::from("Valid citation (Sartre 2021). Incomplete citation Derrida 2021).");
        let valid_citations = extract_citations_from_markdown(&markdown);
        assert_eq!(valid_citations, vec!["Sartre 2021"]);
    }
}

#[cfg(test)]
mod tests_validate_citations {
    use super::*;

    #[test]
    fn valid_citations() {
        let citations = vec!["Hegel 2021".to_string(), "Kant 2020, 123".to_string()];
        assert!(verify_citations_format(&citations).is_ok());
    }
    #[test]
    fn missing_year() {
        let citations = vec!["Hegel".to_string(), "Kant 2020, 123".to_string()];
        assert!(verify_citations_format(&citations).is_err());
    }
    #[test]
    fn invalid_citation_extra_comma() {
        let citations = vec![
            "Hegel 2021".to_string(),
            "Kant 2020, 123".to_string(),
            "Hume, 2020".to_string(),
        ];
        assert!(verify_citations_format(&citations).is_err());
    }
    #[test]
    fn valid_citations_set() {
        let citations = vec![
            "Hegel 2021".to_string(),
            "Kant 2020, 123".to_string(),
            "Hegel 2021".to_string(),
            "Hegel 2021, 1234".to_string(),
            "Hegel 2021, 99".to_string(),
        ];
        let citations_set = create_citations_set(citations);
        assert_eq!(citations_set, vec!["Hegel 2021", "Kant 2020"]);
    }
    #[test]
    fn empty_citations_set() {
        let citations = Vec::<String>::new();
        let citations_set = create_citations_set(citations);
        assert!(citations_set.is_empty());
    }
    #[test]
    fn invalid_citations_set() {
        let citations = vec!["Hegel 2021".to_string(), "Kant, 2020, 123".to_string()];
        let citations_set = create_citations_set(citations);
        assert_eq!(citations_set, vec!["Hegel 2021", "Kant"]);
    }
    // TODO what happened here? investigate
    // #[test]
    // fn test_match_citations_to_bibliography() {
    //     let bibliography = vec![
    //         Entry::new("book", "Hegel 2021"),
    //         Entry::new("book", "Kant 2020"),
    //     ];
    //     let citations = vec!["Hegel 2021".to_string(), "Kant 2020".to_string()];
    //     let matched_citations = match_citations_to_bibliography(citations, &bibliography).unwrap();
    //     assert_eq!(matched_citations, bibliography);
    // }
}
