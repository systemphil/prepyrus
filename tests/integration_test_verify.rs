use prepyrus::{
    cli::{Cli, Mode},
    utils::{Config, LoadOrCreateSettingsTestMode},
    Prepyrus,
};

#[test]
fn run_verify_with_directory() {
    let cli = Cli {
        bib_file: "tests/mocks/test.bib".to_string(),
        target_path: "tests/mocks/data".to_string(),
        mode: Mode::Verify,
        ignore_paths: None,
        generate_index_to_file: None,
        index_link_prefix_rewrite: None,
    };

    let Config {
        bib_file,
        target_path,
        mode,
        settings,
        generate_index_file,
        index_link_prefix_rewrite: link_prefix_rewrite,
    } = Prepyrus::build_config(cli, Some(LoadOrCreateSettingsTestMode::Test)).unwrap_or_else(|e| {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    });

    let all_entries = Prepyrus::get_all_bib_entries(&bib_file).unwrap();
    let mdx_paths = Prepyrus::get_mdx_paths(&target_path, Some(settings.ignore_paths)).unwrap();
    let articles_file_data = Prepyrus::verify(mdx_paths, &all_entries).unwrap();

    println!("{:?}", articles_file_data);
    assert!(mode == Mode::Verify);
    assert!(generate_index_file == None);
    assert!(link_prefix_rewrite == None);
    assert!(articles_file_data.len() > 1);
    assert!(!articles_file_data.is_empty());
}

#[test]
fn run_verify_with_directory_with_ignored_paths_from_settings() {
    let cli = Cli {
        bib_file: "tests/mocks/test.bib".to_string(),
        target_path: "tests/mocks/data".to_string(),
        mode: Mode::Verify,
        ignore_paths: None,
        generate_index_to_file: None,
        index_link_prefix_rewrite: None,
    };

    let Config {
        bib_file,
        target_path,
        mode,
        settings,
        generate_index_file,
        index_link_prefix_rewrite: link_prefix_rewrite,
    } = Prepyrus::build_config(cli, Some(LoadOrCreateSettingsTestMode::Test)).unwrap_or_else(|e| {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    });

    let all_entries = Prepyrus::get_all_bib_entries(&bib_file).unwrap();
    let mdx_paths = Prepyrus::get_mdx_paths(&target_path, Some(settings.ignore_paths)).unwrap();
    let articles_file_data = Prepyrus::verify(mdx_paths, &all_entries).unwrap();

    println!("{:?}", articles_file_data);
    assert!(mode == Mode::Verify);
    assert!(generate_index_file == None);
    assert!(link_prefix_rewrite == None);
    assert!(articles_file_data.len() > 1);
    assert!(!articles_file_data.is_empty());
}

#[test]
fn run_verify_with_directory_with_ignored_paths_from_cli_args() {
    fn run_test(ignored_paths: Vec<String>) {
        let cli = Cli {
            bib_file: "tests/mocks/test.bib".to_string(),
            target_path: "tests/mocks/data".to_string(),
            mode: Mode::Verify,
            ignore_paths: Some(ignored_paths.clone()),
            generate_index_to_file: None,
            index_link_prefix_rewrite: None,
        };

        let Config {
            bib_file,
            target_path,
            mode,
            settings,
            generate_index_file,
            index_link_prefix_rewrite: link_prefix_rewrite,
        } = Prepyrus::build_config(cli, None).unwrap_or_else(|e| {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        });

        let all_entries = Prepyrus::get_all_bib_entries(&bib_file).unwrap();
        let mdx_paths = Prepyrus::get_mdx_paths(&target_path, Some(settings.ignore_paths)).unwrap();
        let articles_file_data = Prepyrus::verify(mdx_paths, &all_entries).unwrap();
        assert!(mode == Mode::Verify);
        assert!(generate_index_file == None);
        assert!(link_prefix_rewrite == None);
        for ignored_path in ignored_paths {
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

    run_test(vec!["tests/mocks/data/development.mdx".into()]);
    run_test(vec![
        "tests/mocks/data/development.mdx".into(),
        "tests/mocks/data/first-paragraph.mdx".into(),
    ]);
}

#[test]
fn run_verify_with_single_file() {
    let cli = Cli {
        bib_file: "tests/mocks/test.bib".to_string(),
        target_path: "tests/mocks/data/science-of-logic-introduction.mdx".to_string(),
        mode: Mode::Verify,
        ignore_paths: None,
        generate_index_to_file: None,
        index_link_prefix_rewrite: None,
    };

    let Config {
        bib_file,
        target_path,
        mode,
        settings,
        generate_index_file,
        index_link_prefix_rewrite: link_prefix_rewrite,
    } = Prepyrus::build_config(cli, Some(LoadOrCreateSettingsTestMode::Test)).unwrap_or_else(|e| {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    });

    let all_entries = Prepyrus::get_all_bib_entries(&bib_file).unwrap();
    let mdx_paths = Prepyrus::get_mdx_paths(&target_path, Some(settings.ignore_paths)).unwrap();
    let articles_file_data = Prepyrus::verify(mdx_paths, &all_entries).unwrap();

    println!("{:?}", articles_file_data);
    assert!(mode == Mode::Verify);
    assert!(generate_index_file == None);
    assert!(link_prefix_rewrite == None);
    assert!(articles_file_data.len() == 1);
    assert!(!articles_file_data.is_empty());
}

#[test]
fn run_process_with_dir() {
    let cli = Cli {
        bib_file: "tests/mocks/test.bib".to_string(),
        target_path: "tests/mocks/data".to_string(),
        mode: Mode::Process,
        ignore_paths: Some(vec!["tests/mocks/data/gen_index.mdx".into()]),
        generate_index_to_file: Some("tests/mocks/data/gen_index.mdx".to_string()),
        index_link_prefix_rewrite: Some(("tests/mocks/data".to_string(), "articles".to_string())),
    };

    let link_prefix_rewrite_arg = cli.index_link_prefix_rewrite.clone();

    let Config {
        bib_file,
        target_path,
        mode,
        settings,
        generate_index_file,
        index_link_prefix_rewrite: link_prefix_rewrite,
    } = Prepyrus::build_config(cli, Some(LoadOrCreateSettingsTestMode::Test)).unwrap_or_else(|e| {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    });

    let all_entries = Prepyrus::get_all_bib_entries(&bib_file).unwrap();
    let mdx_paths = Prepyrus::get_mdx_paths(&target_path, Some(settings.ignore_paths)).unwrap();
    let articles_file_data = Prepyrus::verify(mdx_paths, &all_entries).unwrap();

    println!("{:?}", articles_file_data);
    assert!(mode == Mode::Process);
    assert!(link_prefix_rewrite == Some(("tests/mocks/data".to_string(), "articles".to_string())));
    assert!(generate_index_file.is_some());
    assert!(articles_file_data.len() == 4);
    assert!(!articles_file_data.is_empty());

    Prepyrus::gen_index_to_file(
        articles_file_data.clone(),
        generate_index_file
            .clone()
            .expect("Expected generate_index_file"),
        link_prefix_rewrite_arg.as_ref(),
    );

    Prepyrus::process(articles_file_data);
}
