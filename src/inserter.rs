use biblatex::{Chunk, Entry, EntryType, Spanned};
use regex::Regex;
use std::fs;
use std::io::{self, Write};
use utils::BiblatexUtils;
use validator::{ArticleFileData, Metadata};

use crate::{utils, validator};

struct InserterOutcome {
    total_articles_processed: i32,
    total_bibliographies_inserted: i32,
    total_authors_inserted: i32,
    total_notes_headings_inserted: i32,
    total_empty_payloads: i32,
}

pub fn process_mdx_files(all_articles: Vec<ArticleFileData>) {
    let all_articles_length = all_articles.len();
    let mut inserter_outcome = InserterOutcome {
        total_articles_processed: 0,
        total_bibliographies_inserted: 0,
        total_authors_inserted: 0,
        total_notes_headings_inserted: 0,
        total_empty_payloads: 0,
    };

    for article in all_articles {
        process_mdx_file(article, &mut inserter_outcome);
    }
    println!(
        "===Processing OK. Total articles processed: {}/{}. Inserted {} bibliographies, {} authors, and {} notes headings. {} were empty payloads",
        inserter_outcome.total_articles_processed,
        all_articles_length,
        inserter_outcome.total_bibliographies_inserted,
        inserter_outcome.total_authors_inserted,
        inserter_outcome.total_notes_headings_inserted,
        inserter_outcome.total_empty_payloads
    );
}

fn process_mdx_file(article_file_data: ArticleFileData, inserter_outcome: &mut InserterOutcome) {
    let mut mdx_payload = String::new();
    let mdx_bibliography = generate_mdx_bibliography(article_file_data.matched_citations);

    let mdx_authors = generate_mdx_authors(&article_file_data.metadata);
    let mdx_notes_heading = generate_notes_heading(&article_file_data.markdown_content);

    if !mdx_bibliography.is_empty() {
        mdx_payload.push_str(&mdx_bibliography);
        inserter_outcome.total_bibliographies_inserted += 1;
    }
    if !mdx_authors.is_empty() {
        mdx_payload.push_str(&mdx_authors);
        inserter_outcome.total_authors_inserted += 1;
    }
    if !mdx_notes_heading.is_empty() {
        mdx_payload.push_str(&mdx_notes_heading);
        inserter_outcome.total_notes_headings_inserted += 1;
    }
    if mdx_payload.is_empty() {
        inserter_outcome.total_empty_payloads += 1;
        return;
    }

    let updated_markdown_content =
        format!("{}\n{}", article_file_data.full_file_content, mdx_payload);

    match write_html_to_mdx_file(&article_file_data.path, &updated_markdown_content) {
        Ok(_) => {
            inserter_outcome.total_articles_processed += 1;
            println!(
                "---Success! HTML bibliography inserted for {}",
                article_file_data.path
            );
        }
        Err(err) => {
            eprintln!("Error writing HTML to MDX file: {}", err);
            std::process::exit(1);
        }
    }
}

fn write_html_to_mdx_file(path: &str, content: &str) -> io::Result<()> {
    let file = fs::File::create(path)?;
    let mut writer = io::BufWriter::new(file);
    writer.write_all(content.as_bytes())?;
    Ok(())
}

fn generate_mdx_bibliography(entries: Vec<Entry>) -> String {
    let mut bib_html = String::new();

    if entries.is_empty() {
        return bib_html;
    }

    let mut sorted_entries = entries.clone();
    sorted_entries.sort_by(|a, b| {
        let a_authors = a.author().unwrap_or_default();
        let b_authors = b.author().unwrap_or_default();
        
        let a_last_name = a_authors.first()
            .map(|p| p.name.clone().to_lowercase())
            .unwrap_or_default();
        let b_last_name = b_authors.first()
            .map(|p| p.name.clone().to_lowercase())
            .unwrap_or_default();
        
        a_last_name.cmp(&b_last_name)
    });

    bib_html.push_str("\n## Bibliography\n\n<div className=\"text-sm\">\n");

    for entry in sorted_entries {
        bib_html.push_str("- ");
        match entry.entry_type {
            EntryType::Book => {
                let author = entry.author().unwrap();
                let title_spanned: &[biblatex::Spanned<biblatex::Chunk>] = entry.title().unwrap();
                let title = BiblatexUtils::extract_spanned_chunk(title_spanned);
                let publisher_spanned: Vec<Vec<Spanned<Chunk>>> = entry.publisher().unwrap();
                let publisher = BiblatexUtils::extract_publisher(&publisher_spanned);
                let address_spanned: &[Spanned<Chunk>] = entry.address().unwrap();
                let address = BiblatexUtils::extract_spanned_chunk(address_spanned);
                let date = entry.date().unwrap();
                let year = BiblatexUtils::extract_year(&date, entry.key.clone()).unwrap();
                let translators = entry.translator().unwrap_or(Vec::new());
                let doi = entry.doi().unwrap_or("".to_string());

                add_authors_to_html(author, &mut bib_html);
                bib_html.push_str(&format!("{}. ", year));
                bib_html.push_str(&format!("_{}_. ", title));

                let translators_mdx = generate_contributors(translators, "Translated".to_string());
                if !translators_mdx.is_empty() {
                    bib_html.push_str(&translators_mdx);
                }

                bib_html.push_str(&format!("{}: {}.", address, publisher));

                if !doi.is_empty() {
                    bib_html.push_str(&format!(" https://doi.org/{}.", doi));
                }
            }
            EntryType::Article => {
                let author = entry.author().unwrap();

                let title_spanned: &[biblatex::Spanned<biblatex::Chunk>] = entry.title().unwrap();
                let title = BiblatexUtils::extract_spanned_chunk(title_spanned);
                let journal_spanned = entry.journal().unwrap();
                let journal = BiblatexUtils::extract_spanned_chunk(&journal_spanned);

                let volume_permissive = entry.volume().unwrap();
                let volume = BiblatexUtils::extract_volume(&volume_permissive);

                let number_spanned = entry.number().unwrap();
                let number = BiblatexUtils::extract_spanned_chunk(&number_spanned);

                let pages_permissive = entry.pages().unwrap();
                let pages = BiblatexUtils::extract_pages(&pages_permissive);

                let date = entry.date().unwrap();
                let year = BiblatexUtils::extract_year(&date, entry.key.clone()).unwrap();
                let translators = entry.translator().unwrap_or(Vec::new());

                let doi = entry.doi().unwrap_or("".to_string());

                add_authors_to_html(author, &mut bib_html);
                bib_html.push_str(&format!("\"{}\". ", title));

                bib_html.push_str(&format!(
                    "_{}_ {}, no. {} ({}): {}. ",
                    journal, volume, number, year, pages
                ));

                let translators_mdx = generate_contributors(translators, "Translated".to_string());
                if !translators_mdx.is_empty() {
                    bib_html.push_str(&translators_mdx);
                } 

                if !doi.is_empty() {
                    bib_html.push_str(&format!(" https://doi.org/{}.", doi));
                }
            }
            _ => println!("Entry type not supported: {:?}", entry.entry_type),
        }
        bib_html.push_str("\n");
    }

    bib_html.push_str("</div>\n");

    bib_html = bib_html.replace("..", ".");
    bib_html = bib_html.replace("...", ".");
    bib_html = bib_html.replace("....", ".");

    bib_html
}

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

fn generate_mdx_authors(metadata: &Metadata) -> String {
    let mut mdx_html = String::new();

    if let Some(authors) = &metadata.authors {
        mdx_html.push_str("\n**Authors**  \n");
        mdx_html.push_str(&authors);
        mdx_html.push_str("\n");
    }
    if let Some(editors) = &metadata.editors {
        mdx_html.push_str("\n**Editors**  \n");
        mdx_html.push_str(&editors);
        mdx_html.push_str("\n");
    }
    if let Some(contributors) = &metadata.contributors {
        mdx_html.push_str("\n**Contributors**  \n");
        mdx_html.push_str(&contributors);
        mdx_html.push_str("\n");
    }

    mdx_html
}

fn generate_notes_heading(markdown: &String) -> String {
    let mut mdx_notes_heading = String::new();

    let footnote_regex = Regex::new(r"\[\^1\]").unwrap();

    'outer: for line in markdown.lines() {
        for _captures in footnote_regex.captures_iter(line) {
            mdx_notes_heading.push_str("\n**Notes**");
            break 'outer;
        }
    }
    mdx_notes_heading
}

fn add_authors_to_html(author: Vec<biblatex::Person>, bib_html: &mut String) {
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