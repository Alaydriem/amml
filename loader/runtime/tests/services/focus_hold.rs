use amml_runtime::{FocusHold, FocusMessage};

const KILLFOCUS: FocusMessage = FocusMessage { msg: 0x0008, wparam: 0, lparam: 0 };
const ACTIVATEAPP: FocusMessage = FocusMessage { msg: 0x001C, wparam: 0, lparam: 7 };
const ACTIVATE: FocusMessage = FocusMessage { msg: 0x0006, wparam: 0, lparam: 0 };

#[test]
fn an_uncancelled_loss_passes_and_is_not_held() {
    let hold = FocusHold::new();
    assert!(hold.loss(KILLFOCUS, false));
    assert!(hold.take_replay().is_empty());
}

#[test]
fn a_held_loss_is_replayed_once() {
    let hold = FocusHold::new();
    assert!(!hold.loss(KILLFOCUS, true));
    assert_eq!(hold.take_replay(), vec![KILLFOCUS]);
    assert!(hold.is_replay(&KILLFOCUS));
    assert!(!hold.is_replay(&KILLFOCUS));
    assert!(hold.take_replay().is_empty());
}

#[test]
fn two_held_losses_of_one_kind_replay_once() {
    let hold = FocusHold::new();
    hold.loss(ACTIVATEAPP, true);
    hold.loss(ACTIVATEAPP, true);
    assert_eq!(hold.take_replay(), vec![ACTIVATEAPP]);
    assert!(hold.is_replay(&ACTIVATEAPP));
    assert!(!hold.is_replay(&ACTIVATEAPP));
}

#[test]
fn held_losses_replay_oldest_first() {
    let hold = FocusHold::new();
    hold.loss(ACTIVATE, true);
    hold.loss(KILLFOCUS, true);
    hold.loss(ACTIVATE, true);
    assert_eq!(hold.take_replay(), vec![ACTIVATE, KILLFOCUS]);
}

#[test]
fn a_gain_forgets_a_held_loss() {
    let hold = FocusHold::new();
    hold.loss(KILLFOCUS, true);
    hold.gain();
    assert!(hold.take_replay().is_empty());
}

#[test]
fn a_gain_forgets_a_replay_mark() {
    let hold = FocusHold::new();
    hold.loss(KILLFOCUS, true);
    hold.take_replay();
    hold.gain();
    assert!(!hold.is_replay(&KILLFOCUS));
}

#[test]
fn a_message_that_was_not_replayed_is_not_a_replay() {
    let hold = FocusHold::new();
    hold.loss(KILLFOCUS, true);
    hold.take_replay();
    assert!(!hold.is_replay(&ACTIVATE));
}
