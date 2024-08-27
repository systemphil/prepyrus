use prepyrus::run_prepyrus;

#[test]
fn run_verify_with_directory() {
    let result = run_prepyrus("tests/mocks/test.bib", "tests/mocks/data", "verify");
    println!("{:?}", result);
    assert!(result.is_ok());
}

#[test]
fn run_verify_with_single_file() {
    let result = run_prepyrus(
        "tests/mocks/test.bib",
        "tests/mocks/data/science-of-logic-introduction.mdx",
        "verify",
    );
    println!("{:?}", result);
    assert!(result.is_ok());
}
