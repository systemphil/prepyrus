use itertools::Itertools;
use regex::Regex;
use std::collections::BTreeSet;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use validators::{ArticleFileData, Metadata};

use crate::transformers::transform_keys_to_citations;
use crate::validators::MatchedCitationDisambiguated;
use crate::{transformers, validators};

struct InserterOutcome {
    total_articles_processed: i32,
    total_bibliographies_inserted: i32,
    total_authors_inserted: i32,
    total_notes_headings_inserted: i32,
    total_empty_payloads: i32,
}

#[derive(Debug, Clone)]
pub struct ArticleIndexData {
    /// Path to the file whose contents were extracted.
    pub path: String,

    /// Title for the index entry
    pub index_title: String,
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
        "✓ Processing OK. Total articles processed: {}/{}. Inserted {} bibliographies, {} authors, and {} notes headings. {} were empty payloads",
        inserter_outcome.total_articles_processed,
        all_articles_length,
        inserter_outcome.total_bibliographies_inserted,
        inserter_outcome.total_authors_inserted,
        inserter_outcome.total_notes_headings_inserted,
        inserter_outcome.total_empty_payloads
    );
}

pub fn generate_index_to_file(
    all_articles: Vec<ArticleFileData>,
    index_file_path: String,
    rewrite: Option<&(String, String)>,
) {
    let all_index_data_sorted: Vec<ArticleIndexData> = all_articles
        .into_iter()
        .map(get_index_data)
        .sorted_by(|a, b| {
            a.index_title
                .to_lowercase()
                .cmp(&b.index_title.to_lowercase())
        })
        .collect();

    let mut mdx_html = String::new();
    let mut current_letter: Option<char> = None;
    let mut section_letters = BTreeSet::new();

    for index_data in &all_index_data_sorted {
        let first_letter = index_data
            .index_title
            .chars()
            .next()
            .map(|c| c.to_ascii_uppercase());

        if first_letter != current_letter {
            if let Some(letter) = first_letter {
                mdx_html.push_str(&format!("\n## {}\n", letter));
                section_letters.insert(letter);
                current_letter = Some(letter);
            }
        }

        mdx_html.push_str(generate_index_entry(index_data.clone(), rewrite).as_str());
    }

    let article_count = all_index_data_sorted.len();

    let jump_links = section_letters
        .iter()
        .map(|letter| format!("[{}](#{})", letter, letter.to_ascii_lowercase()))
        .collect::<Vec<_>>()
        .join(" · ");

    let intro = format!(
    "\n_This index contains **{} articles**, organized alphabetically by title. Use the links below to jump to a section:_\n\n{}",
    article_count, jump_links
);

    let final_payload = format!("{}\n\n{}", intro, mdx_html);

    match append_to_file(&index_file_path, &final_payload) {
        Ok(_) => {
            println!("HTML Index inserted for {}", index_file_path);
        }
        Err(err) => {
            eprintln!("Error writing HTML to MDX file: {}", err);
            std::process::exit(1);
        }
    }
}

fn process_mdx_file(article_file_data: ArticleFileData, inserter_outcome: &mut InserterOutcome) {
    let mut mdx_payload = String::new();
    let mdx_bibliography = generate_mdx_bibliography(&article_file_data.entries_disambiguated);

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

    let full_file_content_disambiguated = transform_keys_to_citations(&article_file_data);

    let updated_markdown_content = format!("{}\n{}", full_file_content_disambiguated, mdx_payload);

    match write_html_to_mdx_file(&article_file_data.path, &updated_markdown_content) {
        Ok(_) => {
            inserter_outcome.total_articles_processed += 1;
            println!("HTML bibliography inserted for {}", article_file_data.path);
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

fn append_to_file(path: &str, content: &str) -> std::io::Result<()> {
    let mut file = OpenOptions::new().append(true).create(true).open(path)?;

    writeln!(file, "{}", content)?;
    Ok(())
}

fn generate_mdx_bibliography(entries: &Vec<MatchedCitationDisambiguated>) -> String {
    let mut bib_html = String::new();

    if entries.is_empty() {
        return bib_html;
    }

    let prepared_entries = transformers::entries_to_strings(entries);

    bib_html.push_str("\n## Bibliography\n\n<div className=\"text-sm\">\n");

    for entry in prepared_entries {
        bib_html.push_str("- ");
        bib_html.push_str(&entry);
        bib_html.push_str("\n");
    }

    bib_html.push_str("</div>\n");

    bib_html = bib_html.replace("..", ".");
    bib_html = bib_html.replace("...", ".");
    bib_html = bib_html.replace("....", ".");

    bib_html
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

fn get_index_data(article: ArticleFileData) -> ArticleIndexData {
    ArticleIndexData {
        path: article.path,
        index_title: article.metadata.index_title,
    }
}

fn generate_index_entry(
    index_data: ArticleIndexData,
    rewrite: Option<&(String, String)>,
) -> String {
    let mut link = index_data.path.clone().replace(".mdx", "");

    if let Some((from, to)) = rewrite {
        if link.starts_with(from) {
            link = link.replacen(from, to, 1);
        }
    }

    format!("\n[{}]({})\n", index_data.index_title, link)
}
