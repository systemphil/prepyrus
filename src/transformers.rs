use biblatex::{Entry, EntryType};
use std::collections::HashMap;
use utils::BiblatexUtils;
use validators::{MatchedCitation, MatchedCitationDisambiguated};

use crate::utils;
use crate::validators;

/// Transform a list of entries into a list of strings according to the Chicago bibliography style.
pub fn entries_to_strings(entries: Vec<MatchedCitationDisambiguated>) -> Vec<String> {
    let sorted_entries = sort_entries(entries);
    let mut strings_output: Vec<String> = Vec::new();

    for matched_citation in sorted_entries {
        match matched_citation.entry.entry_type {
            EntryType::Book => {
                strings_output.push(transform_book_entry(&matched_citation));
            }
            EntryType::Article => strings_output.push(transform_article_entry(&matched_citation)),
            _ => println!(
                "Entry type not supported: {:?}",
                &matched_citation.entry.entry_type
            ),
        }
    }

    strings_output
}

/// Transform a book entry into a string according to the Chicago bibliography style.
fn transform_book_entry(matched_citation: &MatchedCitationDisambiguated) -> String {
    let mut book_string = String::new();

    let author = matched_citation.entry.author().unwrap();
    let year = matched_citation.year_disambiguated.clone();
    let title = extract_title(&matched_citation.entry);
    let publisher = extract_publisher(&matched_citation.entry);
    let address = extract_address(&matched_citation.entry);
    let translators = matched_citation.entry.translator().unwrap_or(Vec::new());
    let doi = matched_citation.entry.doi().unwrap_or("".to_string());

    add_authors(author, &mut book_string);
    add_year(year, &mut book_string);
    add_book_title(title, &mut book_string);
    add_translators(translators, &mut book_string);
    add_address_and_publisher(address, publisher, &mut book_string);
    add_doi(doi, &mut book_string);

    book_string.trim_end().to_string()
}

/// Transform an article entry into a string according to the Chicago bibliography style.
fn transform_article_entry(matched_citation: &MatchedCitationDisambiguated) -> String {
    let mut article_string = String::new();

    let author = matched_citation.entry.author().unwrap();
    let year = matched_citation.year_disambiguated.clone();
    let title = extract_title(&matched_citation.entry);
    let journal = extract_journal(&matched_citation.entry);
    let volume = extract_volume(&matched_citation.entry);
    let number = extract_number(&matched_citation.entry);
    let pages = extract_pages(&matched_citation.entry);
    let translators = matched_citation.entry.translator().unwrap_or(Vec::new());
    let doi = matched_citation.entry.doi().unwrap_or("".to_string());

    add_authors(author, &mut article_string);
    add_year(year, &mut article_string);
    add_article_title(title, &mut article_string);
    add_journal_volume_number_pages(journal, volume, number, pages, &mut article_string);
    add_translators(translators, &mut article_string);
    add_doi(doi, &mut article_string);

    article_string.trim_end().to_string()
}

/// Generate a string of a type of contributors.
/// E.g. "Edited", "Translated" become "Edited by", "Translated by".
/// Handles the case when there are multiple contributors.
fn generate_contributors(
    contributors: Vec<biblatex::Person>,
    contributor_description: String,
) -> String {
    let mut contributors_str = String::new();
    if contributors.len() > 1 {
        contributors_str.push_str(&format!("{} by ", contributor_description));
        for (i, person) in contributors.iter().enumerate() {
            if i == contributors.len() - 1 {
                contributors_str.push_str(&format!("and {} {}. ", person.given_name, person.name));
            } else {
                contributors_str.push_str(&format!("{} {}, ", person.given_name, person.name));
            }
        }
    } else if contributors.len() == 1 {
        contributors_str.push_str(&format!(
            "{} by {} {}. ",
            contributor_description, contributors[0].given_name, contributors[0].name
        ));
    }
    contributors_str
}

fn add_year(year: String, target_string: &mut String) {
    target_string.push_str(&format!("{}. ", year));
}

// Adds author(s). Handles multiple.
fn add_authors(author: Vec<biblatex::Person>, bib_html: &mut String) {
    bib_html.push_str(&format_authors(author))
}

///  Returns Chicago style format for authors. Handles the case when there are multiple authors.
fn format_authors(author: Vec<biblatex::Person>) -> String {
    if author.len() > 2 {
        return format!("{}, {} et al. ", author[0].name, author[0].given_name);
    } else if author.len() == 2 {
        // In Chicago style, when listing multiple authors in a bibliography entry,
        // only the first author's name is inverted (i.e., "Last, First"). The second and subsequent
        // authors' names are written in standard order (i.e., "First Last").
        // This rule helps differentiate the primary author from co-authors.
        return format!(
            "{}, {} and {} {}. ",
            author[0].name, author[0].given_name, author[1].given_name, author[1].name
        );
    } else {
        return format!("{}, {}. ", author[0].name, author[0].given_name);
    }
}

/// Add translators to the target string if they exist.
fn add_translators(translators: Vec<biblatex::Person>, target_string: &mut String) {
    let translators_mdx = generate_contributors(translators, "Translated".to_string());
    if !translators_mdx.is_empty() {
        target_string.push_str(&translators_mdx);
    }
}

/// Add DOI to the target string if it exists.
fn add_doi(doi: String, target_string: &mut String) {
    if !doi.is_empty() {
        target_string.push_str(&format!(" https://doi.org/{}.", doi));
    }
}

/// Add book title to the target string. Mainly used for books.
fn add_book_title(title: String, target_string: &mut String) {
    target_string.push_str(&format!("_{}_. ", title));
}

/// Add article title to the target string. Mainly used for articles.
fn add_article_title(title: String, target_string: &mut String) {
    target_string.push_str(&format!("\"{}\". ", title));
}

/// Add address and publisher to the target string. Mainly used for books.
fn add_address_and_publisher(address: String, publisher: String, target_string: &mut String) {
    target_string.push_str(&format!("{}: {}. ", address, publisher));
}

/// Add journal, volume, number, year, and pages to the target string. Mainly used for articles.
fn add_journal_volume_number_pages(
    journal: String,
    volume: i64,
    number: String,
    pages: String,
    target_string: &mut String,
) {
    target_string.push_str(&format!(
        "_{}_ {} ({}): {}. ",
        journal, volume, number, pages
    ));
}

/// Sort entries by author's last name.
fn sort_entries(entries: Vec<MatchedCitationDisambiguated>) -> Vec<MatchedCitationDisambiguated> {
    let mut sorted_entries = entries.clone();
    sorted_entries.sort_by(|a, b| {
        let a_authors = a.entry.author().unwrap_or_default();
        let b_authors = b.entry.author().unwrap_or_default();

        let a_last_name = a_authors
            .first()
            .map(|p| p.name.clone().to_lowercase())
            .unwrap_or_default();
        let b_last_name = b_authors
            .first()
            .map(|p| p.name.clone().to_lowercase())
            .unwrap_or_default();

        a_last_name.cmp(&b_last_name)
    });
    sorted_entries
}

/// Title of the entry.
fn extract_title(entry: &Entry) -> String {
    let title_spanned = entry.title().unwrap();
    let title = BiblatexUtils::extract_spanned_chunk(title_spanned);
    title
}

/// Publisher of the entry.
fn extract_publisher(entry: &Entry) -> String {
    let publisher_spanned = entry.publisher().unwrap();
    let publisher = BiblatexUtils::extract_publisher(&publisher_spanned);
    publisher
}

/// Address of the publisher.
fn extract_address(entry: &Entry) -> String {
    let address_spanned = entry.address().unwrap();
    let address = BiblatexUtils::extract_spanned_chunk(address_spanned);
    address
}

/// Year of entry.
fn extract_date(entry: &Entry) -> i32 {
    let date = entry.date().unwrap();
    let year = BiblatexUtils::extract_year_from_date(&date, entry.key.clone()).unwrap();
    year
}

/// Name of the journal of the article.
fn extract_journal(entry: &Entry) -> String {
    let journal_spanned = entry.journal().unwrap();
    let journal = BiblatexUtils::extract_spanned_chunk(&journal_spanned);
    journal
}

/// Volume of the journal.
fn extract_volume(entry: &Entry) -> i64 {
    let volume_permissive = entry.volume().unwrap();
    let volume = BiblatexUtils::extract_volume(&volume_permissive);
    volume
}

/// Number of the journal.
fn extract_number(entry: &Entry) -> String {
    let number_spanned = entry.number().unwrap();
    let number = BiblatexUtils::extract_spanned_chunk(&number_spanned);
    number
}

/// Pages of the article.
fn extract_pages(entry: &Entry) -> String {
    let pages_permissive = entry.pages().unwrap();
    let pages = BiblatexUtils::extract_pages(&pages_permissive);
    pages
}

/// Transform MatchedCitation vector into MatchedCitationDisambiguated vector
/// This handles all disambiguation logic in one place
pub fn disambiguate_matched_citations(
    citations: Vec<MatchedCitation>,
) -> Vec<MatchedCitationDisambiguated> {
    // Group citations by author-year for disambiguation analysis
    let mut author_year_groups: HashMap<String, Vec<&MatchedCitation>> = HashMap::new();

    for citation in &citations {
        let author = citation.entry.author().unwrap();
        let author_last_name = author[0].name.clone();

        let date = citation.entry.date().unwrap();
        let year =
            BiblatexUtils::extract_year_from_date(&date, citation.entry.key.clone()).unwrap();

        let author_year_key = format!("{}-{}", author_last_name, year);
        author_year_groups
            .entry(author_year_key)
            .or_insert_with(Vec::new)
            .push(citation);
    }

    // Create disambiguation mapping
    let mut citation_to_disambiguated: HashMap<String, String> = HashMap::new();
    let mut year_to_disambiguated: HashMap<String, String> = HashMap::new();

    for (_author_year_key, group_citations) in author_year_groups {
        if group_citations.len() > 1 {
            // Need disambiguation - sort by entry key for consistent ordering
            let mut sorted_citations = group_citations;
            sorted_citations.sort_by(|a, b| a.entry.key.cmp(&b.entry.key));

            for (index, citation) in sorted_citations.iter().enumerate() {
                let letter = char::from(b'a' + index as u8);
                let disambiguated = create_disambiguated_citation(letter, &citation.entry);
                citation_to_disambiguated.insert(citation.citation_raw.clone(), disambiguated);
                let disambiguated_year = create_disambiguated_year(letter, &citation.entry);
                year_to_disambiguated.insert(citation.citation_raw.clone(), disambiguated_year);
            }
        } else {
            // No disambiguation needed - convert to standard format
            let citation = group_citations[0];
            let standard = create_standard_citation(&citation.citation_raw, &citation.entry);
            citation_to_disambiguated.insert(citation.citation_raw.clone(), standard);
        }
    }

    // Transform all citations using the disambiguation map
    citations
        .into_iter()
        .map(|matched_citation| {
            let disambiguated = citation_to_disambiguated
                .get(&matched_citation.citation_raw)
                .cloned()
                .unwrap_or_else(|| matched_citation.citation_raw.clone()); // Fallback

            let disambiguated_year = year_to_disambiguated
                .get(&matched_citation.citation_raw)
                .cloned()
                .unwrap_or_else(|| extract_date(&matched_citation.entry).to_string());

            MatchedCitationDisambiguated {
                citation_raw: matched_citation.citation_raw,
                citation_author_date_disambiguated: disambiguated,
                year_disambiguated: disambiguated_year,
                entry: matched_citation.entry,
            }
        })
        .collect()
}

/// Create disambiguated citation with letter (e.g., "@hegel2020logic, 123" -> "Hegel 2020a")
fn create_disambiguated_citation(letter: char, entry: &Entry) -> String {
    let author = format_authors(entry.author().unwrap());
    let year = extract_date(entry);
    format!("{} {}{}", author, year, letter)
}

fn create_disambiguated_year(letter: char, entry: &Entry) -> String {
    let year = extract_date(entry);
    format!("{}{}", year, letter)
}

/// Create standard citation format (no disambiguation needed)
fn create_standard_citation(raw_citation: &str, entry: &Entry) -> String {
    if raw_citation.starts_with('@') {
        // Convert @key to Author Year format
        let author = entry.author().unwrap();
        let author_last_name = author[0].name.clone();

        let date = entry.date().unwrap();
        let year = BiblatexUtils::extract_year_from_date(&date, entry.key.clone()).unwrap();

        format!("{} {}", author_last_name, year)
    } else {
        // Already in standard format, return as-is
        raw_citation.to_string()
    }
}
