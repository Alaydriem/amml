use std::time::{Duration, Instant};

use amml::LeaveHold;
use amml_abi::AmmlScreen;

#[test]
fn the_title_is_loading_after_a_leave_until_the_save_screen_was_on_top() {
    let hold = LeaveHold::new();
    let left = Instant::now();

    hold.begin(left);

    assert_eq!(hold.settle(AmmlScreen::START, left + Duration::from_millis(100)), AmmlScreen::LOADING);
    assert_eq!(hold.settle(AmmlScreen::LOADING, left + Duration::from_millis(300)), AmmlScreen::LOADING);
    assert_eq!(hold.settle(AmmlScreen::START, left + Duration::from_millis(800)), AmmlScreen::START);
    assert_eq!(hold.settle(AmmlScreen::START, left + Duration::from_millis(900)), AmmlScreen::START);
}

#[test]
fn a_leave_with_no_save_screen_read_gives_the_title_after_the_limit() {
    let hold = LeaveHold::new();
    let left = Instant::now();

    hold.begin(left);

    assert_eq!(hold.settle(AmmlScreen::START, left + LeaveHold::LIMIT - Duration::from_millis(1)), AmmlScreen::LOADING);
    assert_eq!(hold.settle(AmmlScreen::START, left + LeaveHold::LIMIT), AmmlScreen::START);
}

#[test]
fn the_title_is_not_changed_with_no_leave_and_other_screens_are_never_changed() {
    let hold = LeaveHold::new();
    let now = Instant::now();

    assert_eq!(hold.settle(AmmlScreen::START, now), AmmlScreen::START);
    hold.begin(now);
    for screen in [AmmlScreen::TOAST, AmmlScreen::OTHER, AmmlScreen::PAUSE, AmmlScreen::HUD] {
        assert_eq!(hold.settle(screen, now), screen);
    }
    assert_eq!(hold.settle(AmmlScreen::START, now), AmmlScreen::LOADING);
}

#[test]
fn a_new_leave_holds_the_title_again() {
    let hold = LeaveHold::new();
    let first = Instant::now();
    hold.begin(first);
    hold.settle(AmmlScreen::LOADING, first);
    assert_eq!(hold.settle(AmmlScreen::START, first), AmmlScreen::START);

    let second = first + Duration::from_secs(60);
    hold.begin(second);

    assert_eq!(hold.settle(AmmlScreen::START, second), AmmlScreen::LOADING);
}
