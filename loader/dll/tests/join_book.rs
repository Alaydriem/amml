use amml::JoinBook;

#[test]
fn one_join_waits_at_a_time_and_its_answer_is_taken_once() {
    let book = JoinBook::new();

    let first = book.begin(3).unwrap();
    assert_eq!(book.begin(4), None);
    assert_eq!(book.answer(), Some((3, first)));
    assert_eq!(book.answer(), None);
    let second = book.begin(4).unwrap();
    assert_ne!(first, second);
}

#[test]
fn a_cancelled_join_frees_the_book_and_a_cancel_of_another_id_changes_nothing() {
    let book = JoinBook::new();

    let id = book.begin(1).unwrap();
    book.cancel(id + 1);
    assert!(book.is_waiting());
    book.cancel(id);
    assert!(!book.is_waiting());
}
