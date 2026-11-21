use amml::AreaBook;

#[test]
fn a_withdrawn_area_is_no_longer_available() {
    let book = AreaBook::new();
    book.offer("input.key_inject", 1);

    book.withdraw("input.key_inject");

    assert!(!book.is_available("input.key_inject", 1));
    assert!(book.is_available("input.lease", 1));
}
