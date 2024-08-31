use prepyrus::{
    utils::{Config, LoadOrCreateSettingsTestMode},
    Prepyrus,
};

#[test]
fn run_verify_with_directory() {
    let args = vec![
        "tests/mocks/test.bib".to_string(),
        "tests/mocks/data".to_string(),
        "verify".to_string(),
    ];
    let Config {
        bib_file,
        target_path,
        mode,
        settings,
    } = Prepyrus::verify_config(&args, Some(LoadOrCreateSettingsTestMode::Test)).unwrap_or_else(
        |e| {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        },
    );

    let all_entries = Prepyrus::get_all_bib_entries(&bib_file).unwrap();
    let mdx_paths = Prepyrus::get_mdx_paths(&target_path, Some(settings.ignore_paths)).unwrap();
    let articles_file_data = Prepyrus::verify(mdx_paths, &all_entries).unwrap();

    println!("{:?}", articles_file_data);
    assert!(mode == "verify");
    assert!(articles_file_data.len() > 1);
    assert!(!articles_file_data.is_empty());
}

#[test]
fn run_verify_with_directory_with_ignored_paths() {
    let args = vec![
        "tests/mocks/test.bib".to_string(),
        "tests/mocks/data".to_string(),
        "verify".to_string(),
    ];
    let Config {
        bib_file,
        target_path,
        mode,
        settings,
    } = Prepyrus::verify_config(&args, Some(LoadOrCreateSettingsTestMode::Test)).unwrap_or_else(
        |e| {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        },
    );

    let all_entries = Prepyrus::get_all_bib_entries(&bib_file).unwrap();
    let mdx_paths = Prepyrus::get_mdx_paths(&target_path, Some(settings.ignore_paths)).unwrap();
    let articles_file_data = Prepyrus::verify(mdx_paths, &all_entries).unwrap();

    println!("{:?}", articles_file_data);
    assert!(mode == "verify");
    assert!(articles_file_data.len() > 1);
    assert!(!articles_file_data.is_empty());
}

#[test]
fn run_verify_with_single_file() {
    let args = vec![
        "tests/mocks/test.bib".to_string(),
        "tests/mocks/data/science-of-logic-introduction.mdx".to_string(),
        "verify".to_string(),
    ];
    let Config {
        bib_file,
        target_path,
        mode,
        settings,
    } = Prepyrus::verify_config(&args, Some(LoadOrCreateSettingsTestMode::Test)).unwrap_or_else(
        |e| {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        },
    );

    let all_entries = Prepyrus::get_all_bib_entries(&bib_file).unwrap();
    let mdx_paths = Prepyrus::get_mdx_paths(&target_path, Some(settings.ignore_paths)).unwrap();
    let articles_file_data = Prepyrus::verify(mdx_paths, &all_entries).unwrap();

    println!("{:?}", articles_file_data);
    assert!(mode == "verify");
    assert!(articles_file_data.len() == 1);
    assert!(!articles_file_data.is_empty());
}
