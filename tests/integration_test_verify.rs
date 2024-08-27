use prepyrus::run_prepyrus;

#[test]
fn test_prepyrus() {
    let result = run_prepyrus("tests/mocks/test.bib", "tests/mocks/data", "verify");
    println!("{:?}", result);
    assert!(result.is_ok());
}
