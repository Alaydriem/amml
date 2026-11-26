use amml_runtime::{RestoreBook, RestoreFlag};

#[test]
fn the_last_mod_to_release_restores_the_first_value() {
    let book = RestoreBook::new();
    book.set(1, RestoreFlag::HideHud, false);
    book.set(2, RestoreFlag::HideHud, true);

    assert_eq!(book.release_owner(1), []);
    assert_eq!(book.release_owner(2), [(RestoreFlag::HideHud, false)]);
}

#[test]
fn a_mod_that_never_set_restores_nothing() {
    let book = RestoreBook::new();
    book.set(1, RestoreFlag::MouseCaptured, true);

    assert_eq!(book.release_owner(3), []);
}
