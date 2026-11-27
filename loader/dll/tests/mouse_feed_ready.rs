use amml::LoaderState;
use amml::{EightArgMouseFeed, MouseFeed, RealMousePath, SevenArgMouseFeed};
use amml_abi::AmmlStatus;

use crate::support::Harness;

const DEVICE: usize = 0x5000;

// a real feed of the device: a pointer move at 10, 20
fn real_move(path: &RealMousePath) {
    path.feed(DEVICE, RealMousePath::MOVE, 0, 10, 20, 0, 0);
}

fn open_on_this_thread(state: &'static LoaderState) {
    state.events().gate().open();
    state.events().gate().note_game_thread();
}

#[test]
fn injection_is_not_ready_before_the_first_frame() {
    let harness = Harness::start();
    let path = RealMousePath::new(harness.state, DEVICE);
    assert_eq!(path.ready(), Err(AmmlStatus::NOT_READY));
    // a real feed before the first frame decides nothing
    real_move(&path);
    assert_eq!(path.ready(), Err(AmmlStatus::NOT_READY));
}

#[test]
fn injection_is_ready_after_the_first_frame_with_no_real_feed() {
    let harness = Harness::start();
    let path = RealMousePath::new(harness.state, DEVICE);
    open_on_this_thread(harness.state);
    assert_eq!(path.ready(), Ok(()));
    assert!(path.decided());
}

#[test]
fn both_builds_follow_the_same_rule() {
    let harness = Harness::start();
    let eight = EightArgMouseFeed::create(harness.state, DEVICE).unwrap();
    assert_eq!(eight.ready(), Err(AmmlStatus::NOT_READY));
    let seven = SevenArgMouseFeed::create(harness.state, DEVICE).unwrap();
    assert_eq!(seven.ready(), Err(AmmlStatus::NOT_READY));
    open_on_this_thread(harness.state);
    assert_eq!(eight.ready(), Ok(()));
    assert_eq!(seven.ready(), Ok(()));
}

#[test]
fn a_real_feed_on_the_game_thread_keeps_injection_ready() {
    let harness = Harness::start();
    let path = RealMousePath::new(harness.state, DEVICE);
    open_on_this_thread(harness.state);
    assert_eq!(path.ready(), Ok(()));
    real_move(&path);
    assert_eq!(path.ready(), Ok(()));
    assert_eq!(path.last_position(), (10, 20));
}

#[test]
fn a_real_feed_off_the_game_thread_withdraws_injection() {
    let harness = Harness::start();
    let path = std::sync::Arc::new(RealMousePath::new(harness.state, DEVICE));
    open_on_this_thread(harness.state);
    assert_eq!(path.ready(), Ok(()));
    let other = path.clone();
    std::thread::spawn(move || real_move(&other)).join().unwrap();
    assert_eq!(path.ready(), Err(AmmlStatus::NOT_AVAILABLE));
}

#[test]
fn a_first_real_feed_off_the_game_thread_withdraws_injection_too() {
    let harness = Harness::start();
    let path = std::sync::Arc::new(RealMousePath::new(harness.state, DEVICE));
    open_on_this_thread(harness.state);
    let other = path.clone();
    std::thread::spawn(move || real_move(&other)).join().unwrap();
    assert_eq!(path.ready(), Err(AmmlStatus::NOT_AVAILABLE));
}
