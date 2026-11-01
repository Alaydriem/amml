use std::ffi::c_void;

use amml_game::{HeldClient, StopLatch};

#[test]
fn get_gives_the_pointer_until_the_latch_is_raised_and_again_after_it_is_lowered() {
    let mut slot = 0u8;
    let client = (&raw mut slot).cast::<c_void>();
    let held = HeldClient::new();
    let latch = StopLatch::new();

    assert!(held.get(&latch).is_none());

    held.set(client);
    assert_eq!(held.get(&latch).map(|ptr| ptr.as_ptr()), Some(client));

    latch.raise();
    assert!(held.get(&latch).is_none());

    latch.lower();
    assert_eq!(held.get(&latch).map(|ptr| ptr.as_ptr()), Some(client));
}

#[test]
fn clear_forgets_the_pointer() {
    let mut slot = 0u8;
    let held = HeldClient::new();
    let latch = StopLatch::new();

    held.set((&raw mut slot).cast::<c_void>());
    held.clear();

    assert!(held.get(&latch).is_none());
}
