use std::sync::{Arc, Mutex};

use amml_abi::{AmmlReleaseReason, InjectedInputReleased, InputLeaseSuspended};
use amml_api::testing::FakeGame;
use amml_api::{
    ApiError, Area, HookOrder, InjectedInputReleasedExt, InputBlocks, KeyCode, LeaseMode, LeaseRequest, LeaseState,
    Look, MouseButton, MoveIntent,
};

const SHARED: LeaseRequest =
    LeaseRequest { mode: LeaseMode::Shared, blocks: InputBlocks::NONE, passthrough: &[], preempt: false };

const PANEL: LeaseRequest = LeaseRequest {
    mode: LeaseMode::Exclusive,
    blocks: InputBlocks::FULL,
    passthrough: &[KeyCode::ESCAPE],
    preempt: true,
};

// A running game that offers key and mouse injection.
fn running() -> FakeGame {
    let fake = FakeGame::new();
    fake.open_gate();
    fake.offer_area(Area::KEY_INJECT);
    fake.offer_area(Area::MOUSE_INJECT);
    fake
}

#[test]
fn injection_before_the_input_is_bound_is_not_ready() {
    let fake = running();
    let lease = fake.ctx().input().acquire(SHARED).unwrap();

    assert!(matches!(lease.key_down(KeyCode::W), Err(ApiError::NotReady)));
}

#[test]
fn injection_into_an_area_the_build_lacks_is_not_available() {
    let fake = running();
    fake.bind_input();
    let lease = fake.ctx().input().acquire(SHARED).unwrap();

    assert_eq!(lease.turn(Look { yaw: 1.0, pitch: 0.0 }), Err(ApiError::NotAvailable));
}

#[test]
fn a_dropped_hold_releases_its_key() {
    let fake = running();
    fake.bind_input();
    let lease = fake.ctx().input().acquire(SHARED).unwrap();

    let hold = lease.key_down(KeyCode::W).unwrap();
    fake.frame();
    drop(hold);
    fake.frame();

    assert_eq!(fake.injected(), ["key 0x57 down", "key 0x57 up"]);
}

#[test]
fn a_lease_dropped_before_its_frame_sends_nothing() {
    let fake = running();
    fake.bind_input();
    let lease = fake.ctx().input().acquire(SHARED).unwrap();

    let hold = lease.key_down(KeyCode::W).unwrap();
    drop(lease);
    fake.frame();
    drop(hold);
    fake.frame();

    assert!(fake.injected().is_empty());
}

#[test]
fn a_hold_released_before_its_frame_is_one_tap() {
    let fake = running();
    fake.bind_input();
    let lease = fake.ctx().input().acquire(SHARED).unwrap();

    lease.key_down(KeyCode::W).unwrap().release();
    fake.frame();
    fake.frame();

    assert_eq!(fake.injected(), ["key 0x57 down", "key 0x57 up"]);
}

#[test]
fn a_dropped_lease_releases_everything() {
    let fake = running();
    fake.bind_input();
    let lease = fake.ctx().input().acquire(SHARED).unwrap();
    let key = lease.key_down(KeyCode::W).unwrap();
    let button = lease.button_down(MouseButton::Left).unwrap();
    fake.frame();

    drop(lease);
    fake.frame();
    // The holds outlive their lease; releasing them now sends nothing more.
    drop((key, button));
    fake.frame();

    assert_eq!(fake.injected(), ["key 0x57 down", "button left down", "key 0x57 up", "button left up"]);
}

#[test]
fn a_preempt_lease_suspends_a_shared_one() {
    let fake = running();
    fake.offer::<InputLeaseSuspended>();
    fake.bind_input();
    let input = fake.ctx().input();
    let suspended = Arc::new(Mutex::new(Vec::new()));
    let record = suspended.clone();
    let _listener = fake
        .ctx()
        .events()
        .listen::<InputLeaseSuspended>(HookOrder::NORMAL, move |event, _| record.lock().unwrap().push(event.lease))
        .unwrap();
    let shared = input.acquire(SHARED).unwrap();

    let panel = input.acquire(PANEL).unwrap();
    fake.frame();

    assert_eq!(shared.state(), LeaseState::Suspended);
    assert!(matches!(shared.key_down(KeyCode::W), Err(ApiError::Refused)));
    assert_eq!(*suspended.lock().unwrap(), [shared.id()]);

    drop(panel);
    assert_eq!(shared.state(), LeaseState::Active);
}

#[test]
fn turns_are_summed_per_frame_and_a_move_intent_stays_until_changed() {
    let fake = running();
    fake.offer_area(Area::TURN);
    fake.offer_area(Area::MOVE);
    fake.bind_input();
    let lease = fake.ctx().input().acquire(SHARED).unwrap();

    lease.turn(Look { yaw: 60.0, pitch: 0.0 }).unwrap();
    lease.turn(Look { yaw: 30.0, pitch: 0.0 }).unwrap();
    lease.set_move(Some(MoveIntent { forward: 1.0, ..MoveIntent::default() })).unwrap();
    fake.frame();
    fake.frame();
    lease.set_move(None).unwrap();
    fake.frame();

    assert_eq!(fake.injected(), ["turn 90 0", "move forward=1", "move none"]);
}

#[test]
fn a_loader_without_an_input_table_serves_no_input() {
    let fake = FakeGame::with_v1_services();
    fake.open_gate();
    let ctx = fake.ctx();

    assert!(matches!(ctx.input().acquire(SHARED), Err(ApiError::NotAvailable)));
    assert_eq!(ctx.input().is_down(KeyCode::W), None);
    assert!(!ctx.supports(Area::INPUT_LEASE));
}

const EXCLUSIVE: LeaseRequest =
    LeaseRequest { mode: LeaseMode::Exclusive, blocks: InputBlocks::NONE, passthrough: &[], preempt: false };

#[test]
fn an_exclusive_lease_suspends_a_shared_one_until_it_ends() {
    let fake = running();
    let input = fake.ctx().input();
    let shared = input.acquire(SHARED).unwrap();

    let exclusive = input.acquire(EXCLUSIVE).unwrap();
    assert_eq!(shared.state(), LeaseState::Suspended);
    assert_eq!(exclusive.state(), LeaseState::Active);

    drop(exclusive);
    assert_eq!(shared.state(), LeaseState::Active);
}

#[test]
fn a_coexist_pair_keeps_a_shared_lease_active_beside_an_exclusive_one() {
    let fake = running();
    let input = fake.ctx().input();
    input.set_coexist(&[("@fake/mod", "@fake/mod")]).unwrap();
    let shared = input.acquire(SHARED).unwrap();

    let exclusive = input.acquire(EXCLUSIVE).unwrap();

    assert_eq!(shared.state(), LeaseState::Active);
    assert_eq!(exclusive.state(), LeaseState::Active);
    input.set_coexist(&[]).unwrap();
    assert_eq!(shared.state(), LeaseState::Suspended);
}

#[test]
fn a_real_up_that_ends_a_held_key_names_the_key_and_a_new_down_presses_again() {
    let fake = running();
    fake.offer::<InjectedInputReleased>();
    fake.bind_input();
    let released = Arc::new(Mutex::new(Vec::new()));
    let record = released.clone();
    let _listener = fake
        .ctx()
        .events()
        .listen::<InjectedInputReleased>(HookOrder::NORMAL, move |event, _| {
            record.lock().unwrap().push((event.reason, event.released_key(), event.released_button()));
        })
        .unwrap();
    let lease = fake.ctx().input().acquire(SHARED).unwrap();
    let first = lease.key_down(KeyCode::W).unwrap();
    fake.frame();

    fake.real_key(KeyCode::W.0, false);
    fake.frame();
    drop(first);
    let _again = lease.key_down(KeyCode::W).unwrap();
    fake.frame();

    assert_eq!(*released.lock().unwrap(), [(AmmlReleaseReason::RESET, Some(KeyCode::W), None)]);
    assert_eq!(fake.injected(), ["key 0x57 down", "key 0x57 down"]);
}
