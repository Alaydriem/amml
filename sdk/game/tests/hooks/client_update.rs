use std::cell::{Cell, RefCell};
use std::ffi::c_void;
use std::ptr::NonNull;

use amml_game::StopLatch;
use amml_game::hooks::ClientUpdate;

#[test]
fn original_gets_the_arguments_bit_for_bit() {
    let seen = RefCell::new(Vec::new());
    let latch = StopLatch::new();
    let out = ClientUpdate::run(
        |t, i, e| {
            seen.borrow_mut().push((t, i, e));
            7
        },
        0x1000,
        1,
        0x55,
        &latch,
        |_| {},
    );
    assert_eq!(out, 7);
    assert_eq!(*seen.borrow(), vec![(0x1000, 1, 0x55)]);
}

#[test]
fn body_gets_this_when_latch_is_lowered() {
    let latch = StopLatch::new();
    let got = Cell::new(None::<NonNull<c_void>>);
    ClientUpdate::run(|_, _, _| 0, 0x2000, 0, 0, &latch, |c| got.set(Some(c)));
    assert_eq!(got.get().map(|p| p.as_ptr() as usize), Some(0x2000));
}

#[test]
fn raised_latch_skips_the_body() {
    let latch = StopLatch::new();
    latch.raise();
    let calls = Cell::new(0);
    let ran = Cell::new(false);
    let out = ClientUpdate::run(
        |_, _, _| {
            calls.set(calls.get() + 1);
            3
        },
        0x1000,
        1,
        0,
        &latch,
        |_| ran.set(true),
    );
    assert_eq!(out, 3);
    assert_eq!(calls.get(), 1);
    assert!(!ran.get());
}

#[test]
fn null_this_skips_the_body() {
    let latch = StopLatch::new();
    let ran = Cell::new(false);
    ClientUpdate::run(|_, _, _| 0, 0, 1, 0, &latch, |_| ran.set(true));
    assert!(!ran.get());
    assert!(!latch.raised());
}

#[test]
fn panicking_body_raises_latch_and_returns_original_value() {
    let latch = StopLatch::new();
    let calls = Cell::new(0);
    let out = ClientUpdate::run(
        |_, _, _| {
            calls.set(calls.get() + 1);
            9
        },
        0x1000,
        1,
        0,
        &latch,
        |_| panic!("boom"),
    );
    assert_eq!(out, 9);
    assert_eq!(calls.get(), 1);
    assert!(latch.raised());

    let ran = Cell::new(false);
    let again = ClientUpdate::run(|_, _, _| 4, 0x1000, 1, 0, &latch, |_| ran.set(true));
    assert_eq!(again, 4);
    assert!(!ran.get());
}
