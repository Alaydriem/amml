use amml_runtime::FocusReplayBook;

#[test]
fn cancel_marks_a_held_loss_for_that_mod() {
    let book = FocusReplayBook::new();

    book.cancelled(3);

    assert!(book.holds(3));
    assert!(!book.holds(4));
}

#[test]
fn a_gain_forgets_the_held_loss() {
    let book = FocusReplayBook::new();
    book.cancelled(3);
    book.cancelled(4);

    assert!(book.gained());

    assert!(!book.holds(3));
    assert!(!book.holds(4));
    assert!(!book.replay(3));
}

#[test]
fn replay_clears_the_mark() {
    let book = FocusReplayBook::new();
    book.cancelled(3);
    book.cancelled(3);

    assert!(book.replay(3));
    assert!(!book.replay(3));
    assert!(!book.any());
}

#[test]
fn a_replay_of_another_mod_leaves_the_mark() {
    let book = FocusReplayBook::new();
    book.cancelled(3);

    assert!(!book.replay(4));
    assert!(book.holds(3));
}
