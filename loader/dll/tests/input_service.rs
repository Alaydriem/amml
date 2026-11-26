use std::sync::Arc;

use amml::SessionAdapter;
use amml_abi::{AmmlInputBlocks, AmmlLeaseMode, AmmlLeaseState, AmmlMouseButton, AmmlReleaseReason, AmmlStatus};
use amml_runtime::{LeaseTerms, ResourceId};

use crate::support::{Harness, RecordingFeeds, SessionGame};

const W: u16 = 0x57;

// A harness with the session adapter over the stand-in game after one frame, every input area offered
// and the recording feeds bound.
fn start() -> (Harness, Arc<RecordingFeeds>) {
    let harness = Harness::start_with_entries(&SessionGame::entries(&[]));
    SessionAdapter::install(harness.state);
    SessionGame::frame();
    for area in ["input.key_inject", "input.mouse_inject", "input.turn"] {
        harness.state.areas().offer(area, 1);
    }
    let feeds = RecordingFeeds::new();
    feeds.bind(harness.state.input());
    (harness, feeds)
}

fn lease(harness: &Harness, owner: usize, preempt: bool) -> ResourceId {
    let terms =
        LeaseTerms { mode: AmmlLeaseMode::SHARED, blocks: AmmlInputBlocks(0), passthrough: Vec::new(), preempt };
    harness.state.input().acquire(owner, &format!("@test/mod{owner}"), terms).unwrap()
}

#[test]
fn ending_a_lease_releases_holds_once() {
    let (harness, feeds) = start();
    feeds.listen(harness.state.events(), "input.injected_released", 0);
    let input = harness.state.input();
    let lease = lease(&harness, 0, false);
    let hold = input.key_down(lease, W).unwrap();
    SessionGame::frame();

    input.end(lease).unwrap();
    input.end(lease).unwrap();
    input.release_hold(hold).unwrap();
    SessionGame::frame();
    SessionGame::frame();

    assert_eq!(
        feeds.log(),
        [
            "feed key 0x57 down".to_string(),
            format!("event released owner=0 lease={} reason={}", lease.0, AmmlReleaseReason::LEASE_ENDED.0),
            "feed key 0x57 up".to_string(),
        ]
    );
    assert_eq!(input.lease_state(lease), AmmlLeaseState::LOST);
}

#[test]
fn a_preempt_lease_releases_the_holds_of_the_suspended() {
    let (harness, feeds) = start();
    let hub = harness.state.events();
    for event in ["input.injected_released", "input.lease_suspended", "input.lease_resumed"] {
        feeds.listen(hub, event, 0);
    }
    let input = harness.state.input();
    let first = lease(&harness, 0, false);
    input.key_down(first, W).unwrap();
    input.button_down(first, AmmlMouseButton::RIGHT).unwrap();
    SessionGame::frame();

    let preempt = lease(&harness, 1, true);
    SessionGame::frame();
    input.end(preempt).unwrap();
    SessionGame::frame();

    let reason = AmmlReleaseReason::SUSPENDED.0;
    assert_eq!(
        feeds.log(),
        [
            "feed key 0x57 down".to_string(),
            "feed button 1 down".to_string(),
            format!("event suspended owner=0 lease={}", first.0),
            format!("event released owner=0 lease={} reason={reason}", first.0),
            "feed key 0x57 up".to_string(),
            "feed button 1 up".to_string(),
            format!("event resumed owner=0 lease={}", first.0),
        ]
    );
    assert_eq!(input.lease_state(first), AmmlLeaseState::ACTIVE);
}

#[test]
fn a_real_up_from_a_reset_loop_drops_the_queued_injected_up() {
    let (harness, feeds) = start();
    feeds.listen(harness.state.events(), "input.key", 0);
    let input = harness.state.input();
    let lease = lease(&harness, 0, false);
    let hold = input.key_down(lease, W).unwrap();
    SessionGame::frame();

    // The 70.x focus reset feeds an up for every key; the real feed sends it to the game itself.
    for key in 0..0x100u16 {
        input.keys().real(key, false);
    }
    input.release_hold(hold).unwrap();
    SessionGame::frame();

    assert_eq!(feeds.log(), ["event key owner=0 key=0x57 down=1 injected=1 origin=1", "feed key 0x57 down"]);
}

#[test]
fn turns_are_summed_and_fed_with_the_listeners_values() {
    let (harness, feeds) = start();
    feeds.listen(harness.state.events(), "input.look_delta", 0);
    let input = harness.state.input();
    let first = lease(&harness, 0, false);
    let second = lease(&harness, 1, false);
    input.turn(first, 1.0, 2.0).unwrap();
    input.turn(second, 0.5, -1.0).unwrap();
    input.turn(first, 1.0, 0.0).unwrap();

    SessionGame::frame();
    feeds.override_look(3.0, 4.0);
    input.turn(first, 1.0, 1.0).unwrap();
    SessionGame::frame();

    assert_eq!(
        feeds.log(),
        [
            "event look owner=0 yaw=2.5 pitch=1 injected=1 origin=2",
            "feed turn 2.5 1",
            "event look owner=0 yaw=1 pitch=1 injected=1 origin=2",
            "feed turn 3 4",
        ]
    );
}

#[test]
fn a_second_lease_keeps_the_key_down_until_the_last_up() {
    let (harness, feeds) = start();
    let input = harness.state.input();
    let first = lease(&harness, 0, false);
    let second = lease(&harness, 1, false);
    let first_hold = input.key_down(first, W).unwrap();
    let second_hold = input.key_down(second, W).unwrap();
    SessionGame::frame();

    input.release_hold(first_hold).unwrap();
    SessionGame::frame();
    assert_eq!(feeds.log(), ["feed key 0x57 down"]);
    input.release_hold(second_hold).unwrap();
    SessionGame::frame();

    assert_eq!(feeds.log(), ["feed key 0x57 down", "feed key 0x57 up"]);
}

#[test]
fn an_injected_hold_of_a_key_the_user_holds_sends_neither_down_nor_up() {
    let (harness, feeds) = start();
    feeds.listen(harness.state.events(), "input.key", 0);
    let input = harness.state.input();
    input.keys().real(W, true);
    let lease = lease(&harness, 0, false);
    let hold = input.key_down(lease, W).unwrap();
    SessionGame::frame();

    input.release_hold(hold).unwrap();
    SessionGame::frame();

    assert!(feeds.log().is_empty());
    assert_eq!(input.keys().is_down(W), Some(true));
}

#[test]
fn a_second_lease_keeps_the_button_down_until_the_last_up() {
    let (harness, feeds) = start();
    let input = harness.state.input();
    let first = lease(&harness, 0, false);
    let second = lease(&harness, 1, false);
    let first_hold = input.button_down(first, AmmlMouseButton::LEFT).unwrap();
    input.button_down(second, AmmlMouseButton::LEFT).unwrap();
    SessionGame::frame();

    input.release_hold(first_hold).unwrap();
    SessionGame::frame();
    assert_eq!(feeds.log(), ["feed button 0 down"]);
    input.end(second).unwrap();
    SessionGame::frame();

    assert_eq!(feeds.log(), ["feed button 0 down", "feed button 0 up"]);
}

#[test]
fn an_injected_hold_of_a_button_the_user_holds_sends_neither_down_nor_up() {
    let (harness, feeds) = start();
    feeds.listen(harness.state.events(), "input.mouse", 0);
    let input = harness.state.input();
    input.buttons().real(AmmlMouseButton::RIGHT, true);
    let lease = lease(&harness, 0, false);
    let hold = input.button_down(lease, AmmlMouseButton::RIGHT).unwrap();
    SessionGame::frame();

    input.release_hold(hold).unwrap();
    SessionGame::frame();

    assert!(feeds.log().is_empty());
}

#[test]
fn a_real_button_up_during_an_injected_hold_drops_the_queued_injected_up() {
    let (harness, feeds) = start();
    let input = harness.state.input();
    let lease = lease(&harness, 0, false);
    let hold = input.button_down(lease, AmmlMouseButton::MIDDLE).unwrap();
    SessionGame::frame();

    input.buttons().real(AmmlMouseButton::MIDDLE, false);
    input.release_hold(hold).unwrap();
    SessionGame::frame();

    assert_eq!(feeds.log(), ["feed button 2 down"]);
}

#[test]
fn two_holds_of_one_lease_keep_the_key_down_until_both_are_released() {
    let (harness, feeds) = start();
    let input = harness.state.input();
    let lease = lease(&harness, 0, false);
    let first = input.key_down(lease, W).unwrap();
    let second = input.key_down(lease, W).unwrap();
    assert_ne!(first, second);
    SessionGame::frame();

    input.release_hold(first).unwrap();
    SessionGame::frame();
    assert_eq!(feeds.log(), ["feed key 0x57 down"]);
    input.release_hold(second).unwrap();
    SessionGame::frame();

    assert_eq!(feeds.log(), ["feed key 0x57 down", "feed key 0x57 up"]);
}

#[test]
fn a_mod_whose_leases_ended_gets_no_new_lease_until_reopened() {
    let (harness, _feeds) = start();
    let input = harness.state.input();
    let terms = || LeaseTerms {
        mode: AmmlLeaseMode::SHARED,
        blocks: AmmlInputBlocks(0),
        passthrough: Vec::new(),
        preempt: false,
    };

    input.end_owner(0);
    assert_eq!(input.acquire(0, "@test/mod0", terms()), Err(AmmlStatus::REFUSED));
    input.reopen_owner(0);

    assert!(input.acquire(0, "@test/mod0", terms()).is_ok());
}

#[test]
fn a_hold_ended_by_a_real_up_does_not_release_a_newer_key_hold() {
    let (harness, feeds) = start();
    let input = harness.state.input();
    let first = lease(&harness, 0, false);
    let third = lease(&harness, 2, false);
    let first_hold = input.key_down(first, W).unwrap();
    SessionGame::frame();

    input.keys().real(W, true);
    input.keys().real(W, false);
    let third_hold = input.key_down(third, W).unwrap();
    SessionGame::frame();
    input.release_hold(first_hold).unwrap();
    SessionGame::frame();
    assert_eq!(feeds.log(), ["feed key 0x57 down", "feed key 0x57 down"]);

    input.release_hold(third_hold).unwrap();
    SessionGame::frame();
    assert_eq!(feeds.log(), ["feed key 0x57 down", "feed key 0x57 down", "feed key 0x57 up"]);
}

#[test]
fn a_hold_ended_by_a_real_up_does_not_release_a_newer_button_hold() {
    let (harness, feeds) = start();
    let input = harness.state.input();
    let first = lease(&harness, 0, false);
    let third = lease(&harness, 2, false);
    let first_hold = input.button_down(first, AmmlMouseButton::LEFT).unwrap();
    SessionGame::frame();

    input.buttons().real(AmmlMouseButton::LEFT, true);
    input.buttons().real(AmmlMouseButton::LEFT, false);
    let third_hold = input.button_down(third, AmmlMouseButton::LEFT).unwrap();
    SessionGame::frame();
    input.release_hold(first_hold).unwrap();
    SessionGame::frame();
    assert_eq!(feeds.log(), ["feed button 0 down", "feed button 0 down"]);

    input.release_hold(third_hold).unwrap();
    SessionGame::frame();
    assert_eq!(feeds.log(), ["feed button 0 down", "feed button 0 down", "feed button 0 up"]);
}

#[test]
fn a_key_pressed_again_after_a_real_up_reaches_the_game() {
    let (harness, feeds) = start();
    let input = harness.state.input();
    let lease = lease(&harness, 0, false);
    let first = input.key_down(lease, W).unwrap();
    SessionGame::frame();

    input.keys().real(W, true);
    input.keys().finish(W, true, false);
    input.keys().real(W, false);
    let second = input.key_down(lease, W).unwrap();
    SessionGame::frame();
    assert_eq!(feeds.log(), ["feed key 0x57 down", "feed key 0x57 down"]);

    input.release_hold(first).unwrap();
    SessionGame::frame();
    assert_eq!(feeds.log().len(), 2);
    input.release_hold(second).unwrap();
    SessionGame::frame();
    assert_eq!(feeds.log(), ["feed key 0x57 down", "feed key 0x57 down", "feed key 0x57 up"]);
}

#[test]
fn a_button_pressed_again_after_a_real_up_reaches_the_game() {
    let (harness, feeds) = start();
    let input = harness.state.input();
    let lease = lease(&harness, 0, false);
    let first = input.button_down(lease, AmmlMouseButton::LEFT).unwrap();
    SessionGame::frame();

    input.buttons().real(AmmlMouseButton::LEFT, true);
    input.buttons().real(AmmlMouseButton::LEFT, false);
    let second = input.button_down(lease, AmmlMouseButton::LEFT).unwrap();
    SessionGame::frame();
    assert_eq!(feeds.log(), ["feed button 0 down", "feed button 0 down"]);

    input.release_hold(first).unwrap();
    input.release_hold(second).unwrap();
    SessionGame::frame();
    assert_eq!(feeds.log(), ["feed button 0 down", "feed button 0 down", "feed button 0 up"]);
}

#[test]
fn a_key_pressed_twice_in_one_frame_reaches_the_game_once() {
    let (harness, feeds) = start();
    let input = harness.state.input();
    let lease = lease(&harness, 0, false);
    input.key_down(lease, W).unwrap();
    input.key_down(lease, W).unwrap();
    SessionGame::frame();

    assert_eq!(feeds.log(), ["feed key 0x57 down"]);
}

#[test]
fn a_real_up_that_ends_an_injected_hold_tells_the_owner_and_a_new_down_presses_again() {
    let (harness, feeds) = start();
    feeds.listen(harness.state.events(), "input.injected_released", 0);
    let input = harness.state.input();
    let lease = lease(&harness, 0, false);
    let shift = input.key_down(lease, 0x10).unwrap();
    input.key_down(lease, W).unwrap();
    SessionGame::frame();

    input.real_key(W, false);
    SessionGame::frame();
    input.key_down(lease, W).unwrap();
    SessionGame::frame();
    input.release_hold(shift).unwrap();
    SessionGame::frame();

    let reason = AmmlReleaseReason::RESET.0;
    assert_eq!(
        feeds.log(),
        [
            "feed key 0x10 down".to_string(),
            "feed key 0x57 down".to_string(),
            format!("event released owner=0 lease={} reason={reason} input=1 code=0x57", lease.0),
            "feed key 0x57 down".to_string(),
            "feed key 0x10 up".to_string(),
        ]
    );
}

#[test]
fn a_real_button_up_that_ends_an_injected_hold_tells_the_owner() {
    let (harness, feeds) = start();
    feeds.listen(harness.state.events(), "input.injected_released", 0);
    let input = harness.state.input();
    let lease = lease(&harness, 0, false);
    input.button_down(lease, AmmlMouseButton::RIGHT).unwrap();
    SessionGame::frame();

    input.real_button(AmmlMouseButton::RIGHT, false);
    input.real_button(AmmlMouseButton::RIGHT, false);
    SessionGame::frame();

    let reason = AmmlReleaseReason::RESET.0;
    assert_eq!(
        feeds.log(),
        [
            "feed button 1 down".to_string(),
            format!("event released owner=0 lease={} reason={reason} input=2 code=0x1", lease.0),
        ]
    );
}

#[test]
fn an_exclusive_lease_suspends_a_shared_one_unless_the_pair_coexists() {
    let (harness, feeds) = start();
    let hub = harness.state.events();
    for event in ["input.lease_suspended", "input.lease_resumed"] {
        feeds.listen(hub, event, 0);
    }
    let input = harness.state.input();
    let shared = lease(&harness, 0, false);
    let exclusive = LeaseTerms {
        mode: AmmlLeaseMode::EXCLUSIVE,
        blocks: AmmlInputBlocks(0),
        passthrough: Vec::new(),
        preempt: false,
    };
    let winner = input.acquire(1, "@test/mod1", exclusive).unwrap();
    SessionGame::frame();
    input.set_coexist(1, vec![("@test/mod0".to_string(), "@test/mod1".to_string())]).unwrap();
    SessionGame::frame();
    assert_eq!(input.set_coexist(0, Vec::new()), Err(AmmlStatus::REFUSED));
    input.end(winner).unwrap();
    SessionGame::frame();

    assert_eq!(
        feeds.log(),
        [format!("event suspended owner=0 lease={}", shared.0), format!("event resumed owner=0 lease={}", shared.0)]
    );
    assert_eq!(input.lease_state(shared), AmmlLeaseState::ACTIVE);
}

#[test]
fn a_focus_loss_after_the_reset_loop_of_the_same_frame_still_tells_the_owner() {
    let (harness, feeds) = start();
    feeds.listen(harness.state.events(), "input.injected_released", 0);
    let input = harness.state.input();
    let lease = lease(&harness, 0, false);
    input.key_down(lease, W).unwrap();
    SessionGame::frame();

    // The 70.x focus path feeds an up for every key, then the loader releases for the focus loss
    for key in 0..0x100u16 {
        input.real_key(key, false);
    }
    input.release_everything(AmmlReleaseReason::FOCUS_LOST);
    SessionGame::frame();
    input.release_everything(AmmlReleaseReason::FOCUS_LOST);
    SessionGame::frame();

    assert_eq!(
        feeds.log(),
        [
            "feed key 0x57 down".to_string(),
            format!("event released owner=0 lease={} reason={} input=1 code=0x57", lease.0, AmmlReleaseReason::RESET.0),
            format!("event released owner=0 lease={} reason={}", lease.0, AmmlReleaseReason::FOCUS_LOST.0),
        ]
    );
}
