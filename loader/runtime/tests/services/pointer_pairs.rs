use amml_abi::AmmlMouseButton;
use amml_runtime::PointerPairs;

#[test]
fn a_cancelled_window_down_cancels_its_up_and_both_feed_copies() {
    let pairs = PointerPairs::new();
    assert!(!pairs.window_button(AmmlMouseButton::LEFT, true, true));
    assert!(!pairs.feed_button(AmmlMouseButton::LEFT, true));
    assert!(!pairs.window_button(AmmlMouseButton::LEFT, false, false));
    assert!(!pairs.feed_button(AmmlMouseButton::LEFT, false));
    assert!(pairs.window_button(AmmlMouseButton::LEFT, true, false));
    assert!(pairs.feed_button(AmmlMouseButton::LEFT, true));
    assert!(pairs.window_button(AmmlMouseButton::LEFT, false, false));
    assert!(pairs.feed_button(AmmlMouseButton::LEFT, false));
}

#[test]
fn a_cancel_of_one_button_leaves_the_others_alone() {
    let pairs = PointerPairs::new();
    assert!(!pairs.window_button(AmmlMouseButton::LEFT, true, true));
    assert!(pairs.window_button(AmmlMouseButton::RIGHT, true, false));
    assert!(pairs.feed_button(AmmlMouseButton::RIGHT, true));
}

#[test]
fn a_cancelled_up_of_a_passed_down_still_reaches_the_game() {
    let pairs = PointerPairs::new();
    assert!(pairs.window_button(AmmlMouseButton::LEFT, true, false));
    assert!(pairs.window_button(AmmlMouseButton::LEFT, false, true));
}

#[test]
fn an_up_whose_down_was_never_seen_reaches_the_game() {
    let pairs = PointerPairs::new();
    assert!(pairs.window_button(AmmlMouseButton::RIGHT, false, true));
}

#[test]
fn a_new_down_ends_the_pairing_of_a_feed_up_that_never_came() {
    let pairs = PointerPairs::new();
    assert!(!pairs.window_button(AmmlMouseButton::LEFT, true, true));
    assert!(pairs.window_button(AmmlMouseButton::LEFT, true, false));
    assert!(pairs.feed_button(AmmlMouseButton::LEFT, true));
}

#[test]
fn an_unknown_button_follows_its_cancel_and_feeds_on() {
    let pairs = PointerPairs::new();
    assert!(!pairs.window_button(AmmlMouseButton(9), true, true));
    assert!(pairs.window_button(AmmlMouseButton(9), true, false));
    assert!(pairs.feed_button(AmmlMouseButton(9), true));
}
