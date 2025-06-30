use biblatex::{Entry, EntryType};
use std::collections::HashMap;
use utils::BiblatexUtils;

use crate::utils;

/// Transform a list of entries into a list of strings according to the Chicago bibliography style.
pub fn entries_to_strings(entries: Vec<Entry>) -> Vec<String> {
    let entries_clone = entries.clone();
    let sorted_entries = sort_entries(entries);
    let mut strings_output: Vec<String> = Vec::new();

    for entry in sorted_entries {
        match entry.entry_type {
            EntryType::Book => {
                strings_output.push(transform_book_entry(&entry, entries_clone.clone()));
            }
            EntryType::Article => strings_output.push(transform_article_entry(&entry)),
            _ => println!("Entry type not supported: {:?}", entry.entry_type),
        }
    }

    strings_output
}

/// Transform a book entry into a string according to the Chicago bibliography style.
fn transform_book_entry(entry: &Entry, entries: Vec<Entry>) -> String {
    let mut book_string = String::new();

    let author = entry.author().unwrap();
    let title = extract_title(entry);
    let publisher = extract_publisher(entry);
    let address = extract_address(entry);
    let year = extract_date_with_context(entry, entries);
    let translators = entry.translator().unwrap_or(Vec::new());
    let doi = entry.doi().unwrap_or("".to_string());

    add_authors(author, &mut book_string);
    add_year(year, &mut book_string);
    add_book_title(title, &mut book_string);
    add_translators(translators, &mut book_string);
    add_address_and_publisher(address, publisher, &mut book_string);
    add_doi(doi, &mut book_string);

    book_string.trim_end().to_string()
}

/// Transform an article entry into a string according to the Chicago bibliography style.
fn transform_article_entry(entry: &Entry) -> String {
    let mut article_string = String::new();

    let author = entry.author().unwrap();
    let title = extract_title(entry);
    let journal = extract_journal(entry);
    let volume = extract_volume(entry);
    let number = extract_number(entry);
    let pages = extract_pages(entry);
    let year = extract_date(entry);
    let translators = entry.translator().unwrap_or(Vec::new());
    let doi = entry.doi().unwrap_or("".to_string());

    add_authors(author, &mut article_string);
    add_article_title(title, &mut article_string);
    add_journal_volume_number_year_pages(journal, volume, number, year, pages, &mut article_string);
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

/// Add authors to the target string. Handles the case when there are multiple authors.
fn add_authors(author: Vec<biblatex::Person>, bib_html: &mut String) {
    if author.len() > 2 {
        bib_html.push_str(&format!(
            "{}, {} et al. ",
            author[0].name, author[0].given_name
        ));
    } else if author.len() == 2 {
        // In Chicago style, when listing multiple authors in a bibliography entry,
        // only the first author's name is inverted (i.e., "Last, First"). The second and subsequent
        // authors' names are written in standard order (i.e., "First Last").
        // This rule helps differentiate the primary author from co-authors.
        bib_html.push_str(&format!(
            "{}, {} and {} {}. ",
            author[0].name, author[0].given_name, author[1].given_name, author[1].name
        ));
    } else {
        bib_html.push_str(&format!("{}, {}. ", author[0].name, author[0].given_name));
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

/// Add year to the target string.
fn add_year(year: String, target_string: &mut String) {
    target_string.push_str(&format!("{}. ", year));
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
fn add_journal_volume_number_year_pages(
    journal: String,
    volume: i64,
    number: String,
    year: i32,
    pages: String,
    target_string: &mut String,
) {
    target_string.push_str(&format!(
        "_{}_ {}, no. {} ({}): {}. ",
        journal, volume, number, year, pages
    ));
}

/// Sort entries by author's last name.
fn sort_entries(entries: Vec<Entry>) -> Vec<Entry> {
    let mut sorted_entries = entries.clone();
    sorted_entries.sort_by(|a, b| {
        let a_authors = a.author().unwrap_or_default();
        let b_authors = b.author().unwrap_or_default();

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

/// Extracts year with disambiguation letter if needed
/// Returns the year as a string with letter suffix (e.g., "1991a", "1991b") for disambiguation
fn extract_date_with_disambiguation(entries: Vec<Entry>) -> HashMap<String, String> {
    let mut year_map = HashMap::new();
    let mut author_year_counts: HashMap<String, Vec<String>> = HashMap::new();

    // First pass: group entries by author-year combination
    for entry in &entries {
        let author = entry.author().unwrap();
        let author_last_name = author[0].name.clone();

        let date = entry.date().unwrap();
        let year = BiblatexUtils::extract_year_from_date(&date, entry.key.clone()).unwrap();

        let author_year_key = format!("{}-{}", author_last_name, year);
        let entry_key = entry.key.clone();

        author_year_counts
            .entry(author_year_key)
            .or_insert_with(Vec::new)
            .push(entry_key);
    }

    // Second pass: assign disambiguation letters
    for entry in &entries {
        let author = entry.author().unwrap();
        let author_last_name = author[0].name.clone();

        let date = entry.date().unwrap();
        let year = BiblatexUtils::extract_year_from_date(&date, entry.key.clone()).unwrap();

        let author_year_key = format!("{}-{}", author_last_name, year);
        let entry_key = entry.key.clone();

        let entries_for_author_year = author_year_counts.get(&author_year_key).unwrap();

        let disambiguated_year = if entries_for_author_year.len() > 1 {
            // Multiple entries for same author-year, add letter
            let position = entries_for_author_year
                .iter()
                .position(|k| k == &entry_key)
                .unwrap();
            let letter = char::from(b'a' + position as u8);
            format!("{}{}", year, letter)
        } else {
            // Only one entry for this author-year, no disambiguation needed
            year.to_string()
        };

        year_map.insert(entry_key, disambiguated_year);
    }

    year_map
}

/// Updated extract_date function that uses disambiguation
fn extract_date_with_context(entry: &Entry, entries: Vec<Entry>) -> String {
    let disambiguation_map = extract_date_with_disambiguation(entries);
    let key = entry.key.clone();

    disambiguation_map
        .get(&key)
        .unwrap_or(&"Unknown".to_string())
        .clone()
}
