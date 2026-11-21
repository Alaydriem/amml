//! The "Input holds" rules of `docs/api.md` section 2, checked against `FakeGame`. Each test is named
//! after the rule it checks; the fake runs the loader's key table and lease book.

use amml_api::testing::FakeGame;
use amml_api::{Area, InputBlocks, KeyCode, LeaseMode, LeaseRequest};

const SHARED: LeaseRequest =
    LeaseRequest { mode: LeaseMode::Shared, blocks: InputBlocks::NONE, passthrough: &[], preempt: false };

const BLOCK_KEYS: LeaseRequest =
    LeaseRequest { mode: LeaseMode::Shared, blocks: InputBlocks::KEYS, passthrough: &[], preempt: false };

const W: u16 = 0x57;

// A running game with key injection offered and the input feeds bound.
fn running() -> FakeGame {
    let fake = FakeGame::new();
    fake.open_gate();
    fake.offer_area(Area::KEY_INJECT);
    fake.bind_input();
    fake
}

#[test]
fn two_holds_of_one_key_send_one_down_and_one_up() {
    let fake = running();
    let input = fake.ctx().input();
    let first = input.acquire(SHARED).unwrap();
    let second = input.acquire(SHARED).unwrap();

    let a = first.key_down(KeyCode::W).unwrap();
    let b = first.key_down(KeyCode::W).unwrap();
    let c = second.key_down(KeyCode::W).unwrap();
    fake.frame();
    drop(a);
    fake.frame();
    drop(c);
    fake.frame();
    assert_eq!(fake.injected(), ["key 0x57 down"], "a hold of the same lease or another lease keeps the key down");
    drop(b);
    fake.frame();

    assert_eq!(fake.injected(), ["key 0x57 down", "key 0x57 up"]);
}

#[test]
fn a_real_up_ends_every_injected_hold_of_that_key() {
    let fake = running();
    let input = fake.ctx().input();
    let first = input.acquire(SHARED).unwrap();
    let second = input.acquire(SHARED).unwrap();
    let a = first.key_down(KeyCode::W).unwrap();
    let b = second.key_down(KeyCode::W).unwrap();
    fake.frame();

    assert!(fake.real_key(W, true));
    assert!(fake.real_key(W, false), "the real up reaches the game");
    drop((a, b));
    fake.frame();

    assert_eq!(fake.injected(), ["key 0x57 down"], "the injected ups after the real up are dropped");
}

#[test]
fn an_injected_key_the_user_holds_sends_no_down_and_no_up() {
    let fake = running();
    let lease = fake.ctx().input().acquire(SHARED).unwrap();

    assert!(fake.real_key(W, true));
    let hold = lease.key_down(KeyCode::W).unwrap();
    fake.frame();
    drop(hold);
    fake.frame();

    assert!(fake.injected().is_empty());
    assert!(fake.real_key(W, false), "the user's up still reaches the game");
}

#[test]
fn a_blocked_real_press_holds_nothing() {
    let fake = running();
    let input = fake.ctx().input();
    let _blocker = input.acquire(BLOCK_KEYS).unwrap();
    let lease = input.acquire(SHARED).unwrap();

    assert!(!fake.real_key(W, true), "the lease blocks the real down");
    let hold = lease.key_down(KeyCode::W).unwrap();
    fake.frame();
    assert_eq!(fake.injected(), ["key 0x57 down"], "the injected hold sends its own down");
    assert!(!fake.real_key(W, false), "the up of a held-back press is held back");
    drop(hold);
    fake.frame();

    assert_eq!(fake.injected(), ["key 0x57 down", "key 0x57 up"], "the held-back real up ended no injected hold");
}

#[test]
fn a_new_key_down_after_a_real_up_sends_a_new_down() {
    let fake = running();
    let lease = fake.ctx().input().acquire(SHARED).unwrap();
    let first = lease.key_down(KeyCode::W).unwrap();
    fake.frame();
    assert!(fake.real_key(W, true));
    assert!(fake.real_key(W, false));

    let second = lease.key_down(KeyCode::W).unwrap();
    fake.frame();
    drop(first);
    fake.frame();
    assert_eq!(fake.injected(), ["key 0x57 down", "key 0x57 down"], "the ended hold sends no up");
    drop(second);
    fake.frame();

    assert_eq!(fake.injected(), ["key 0x57 down", "key 0x57 down", "key 0x57 up"]);
}
