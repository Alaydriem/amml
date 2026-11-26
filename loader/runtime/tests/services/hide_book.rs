use amml_runtime::HideBook;

const FIRST: usize = 1;
const SECOND: usize = 2;

#[test]
fn any_wish_hides() {
    let book = HideBook::new();
    assert!(!book.hidden());

    book.set(FIRST, true);
    book.set(SECOND, true);
    assert!(book.hidden());

    // One mod withdraws its wish; the other still hides the body.
    book.set(FIRST, false);
    assert!(book.hidden());

    book.set(SECOND, false);
    assert!(!book.hidden());
}

#[test]
fn a_repeated_wish_counts_once() {
    let book = HideBook::new();
    book.set(FIRST, true);
    book.set(FIRST, true);
    book.set(FIRST, false);
    assert!(!book.hidden());
}

#[test]
fn disable_clears_only_the_wish_of_that_mod() {
    let book = HideBook::new();
    book.set(FIRST, true);
    book.set(SECOND, true);

    assert!(book.disable(FIRST));
    assert!(book.hidden());
    assert!(!book.disable(FIRST));

    assert!(book.disable(SECOND));
    assert!(!book.hidden());
}

#[test]
fn world_left_clears_the_hide() {
    let book = HideBook::new();
    book.set(FIRST, true);
    book.set(SECOND, true);

    assert!(book.world_left());
    assert!(!book.hidden());
    // A leave with nothing hidden changes nothing.
    assert!(!book.world_left());
}
