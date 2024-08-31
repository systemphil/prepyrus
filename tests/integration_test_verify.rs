use prepyrus::{
    utils::{Config, LoadOrCreateSettingsTestMode},
    Prepyrus,
};

#[test]
fn run_verify_with_directory() {
    let args = vec![
        "program_index".to_string(),
        "tests/mocks/test.bib".to_string(),
        "tests/mocks/data".to_string(),
        "verify".to_string(),
    ];
    let Config {
        bib_file,
        target_path,
        mode,
        settings,
    } = Prepyrus::build_config(&args, Some(LoadOrCreateSettingsTestMode::Test)).unwrap_or_else(
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
fn run_verify_with_directory_with_ignored_paths_from_settings() {
    let args = vec![
        "program_index".to_string(),
        "tests/mocks/test.bib".to_string(),
        "tests/mocks/data".to_string(),
        "verify".to_string(),
    ];
    let Config {
        bib_file,
        target_path,
        mode,
        settings,
    } = Prepyrus::build_config(&args, Some(LoadOrCreateSettingsTestMode::Test)).unwrap_or_else(
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
fn run_verify_with_directory_with_ignored_paths_from_cli_args() {
    fn run_test(ignored_paths: &str) {
        let args = vec![
            "program_index".to_string(),
            "tests/mocks/test.bib".to_string(),
            "tests/mocks/data".to_string(),
            "verify".to_string(),
            ignored_paths.to_string(),
        ];
        let Config {
            bib_file,
            target_path,
            mode,
            settings,
        } = Prepyrus::build_config(&args, None).unwrap_or_else(|e| {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        });

        let all_entries = Prepyrus::get_all_bib_entries(&bib_file).unwrap();
        let mdx_paths = Prepyrus::get_mdx_paths(&target_path, Some(settings.ignore_paths)).unwrap();
        let articles_file_data = Prepyrus::verify(mdx_paths, &all_entries).unwrap();
        let ignored_paths_vec: Vec<String> =
            ignored_paths.split(',').map(|s| s.to_string()).collect();
        assert!(mode == "verify");
        for ignored_path in &ignored_paths_vec {
            assert!(
                articles_file_data
                    .iter()
                    .find(|article| article.path == *ignored_path)
                    .is_none(),
                "Article with the path '{}' found",
                ignored_path
            );
        }
        assert!(articles_file_data.len() >= 1);
        assert!(!articles_file_data.is_empty());
    }

    run_test("tests/mocks/data/development.mdx");
    run_test("tests/mocks/data/development.mdx,tests/mocks/data/first-paragraph.mdx");
}

#[test]
fn run_verify_with_single_file() {
    let args = vec![
        "program_index".to_string(),
        "tests/mocks/test.bib".to_string(),
        "tests/mocks/data/science-of-logic-introduction.mdx".to_string(),
        "verify".to_string(),
    ];
    let Config {
        bib_file,
        target_path,
        mode,
        settings,
    } = Prepyrus::build_config(&args, Some(LoadOrCreateSettingsTestMode::Test)).unwrap_or_else(
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
