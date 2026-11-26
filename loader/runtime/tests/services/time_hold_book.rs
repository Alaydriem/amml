use std::time::{Duration, Instant};

use amml_abi::AmmlTimeHold;
use amml_runtime::{TimeHoldBook, TimeHoldRefusal};

const OWNER: usize = 1;
const OTHER: usize = 2;
const WATCHDOG_MS: u32 = 500;

fn held(now: Instant) -> (TimeHoldBook, AmmlTimeHold) {
    let mut book = TimeHoldBook::new();
    let hold = book.hold(OWNER, WATCHDOG_MS, now).unwrap();
    (book, hold)
}

#[test]
fn second_holder_is_refused() {
    let now = Instant::now();
    let (mut book, hold) = held(now);

    assert_eq!(book.hold(OTHER, WATCHDOG_MS, now), Err(TimeHoldRefusal::Taken));
    // One holder for the process: the holder itself cannot take a second hold either.
    assert_eq!(book.hold(OWNER, WATCHDOG_MS, now), Err(TimeHoldRefusal::Taken));
    assert_eq!(book.holder(), Some(OWNER));

    assert_eq!(book.release(hold), Ok(()));
    let next = book.hold(OTHER, WATCHDOG_MS, now).unwrap();
    // The old handle is stale: it neither steps nor releases the new hold.
    assert_ne!(next, hold);
    assert_eq!(book.step(hold, now), Err(TimeHoldRefusal::Stale));
    assert_eq!(book.release(hold), Err(TimeHoldRefusal::Stale));
    assert_eq!(book.holder(), Some(OTHER));
}

#[test]
fn watchdog_releases_an_idle_hold() {
    let start = Instant::now();
    let (mut book, hold) = held(start);
    let at = |ms: u64| start + Duration::from_millis(ms);

    // Each step feeds the watchdog.
    assert_eq!(book.watchdog(at(400)), None);
    assert_eq!(book.step(hold, at(400)), Ok(()));
    assert_eq!(book.watchdog(at(800)), None);

    // No step for longer than `watchdog_ms`: the hold is released.
    assert_eq!(book.watchdog(at(901)), Some(OWNER));
    assert_eq!(book.holder(), None);
    assert_eq!(book.step(hold, at(902)), Err(TimeHoldRefusal::Stale));
    assert_eq!(book.watchdog(at(5000)), None);
}

#[test]
fn a_zero_watchdog_takes_the_default() {
    let start = Instant::now();
    let mut book = TimeHoldBook::new();
    book.hold(OWNER, 0, start).unwrap();

    assert_eq!(book.watchdog(start + TimeHoldBook::DEFAULT_WATCHDOG), None);
    assert_eq!(book.watchdog(start + TimeHoldBook::DEFAULT_WATCHDOG + Duration::from_millis(1)), Some(OWNER));
}

#[test]
fn disable_releases_the_hold() {
    let now = Instant::now();
    let (mut book, hold) = held(now);

    // Another mod's disable changes nothing.
    assert!(!book.disable(OTHER));
    assert_eq!(book.holder(), Some(OWNER));

    assert!(book.disable(OWNER));
    assert_eq!(book.holder(), None);
    assert_eq!(book.step(hold, now), Err(TimeHoldRefusal::Stale));
    assert_eq!(book.hold(OTHER, WATCHDOG_MS, now).map(|_| ()), Ok(()));
}

#[test]
fn world_left_releases_the_hold() {
    let now = Instant::now();
    let (mut book, hold) = held(now);

    assert_eq!(book.world_left(), Some(OWNER));
    assert_eq!(book.holder(), None);
    assert_eq!(book.release(hold), Err(TimeHoldRefusal::Stale));
    // A world leave with nothing held releases nothing.
    assert_eq!(book.world_left(), None);
}
