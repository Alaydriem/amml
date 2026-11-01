use amml_game::StopLatch;

#[test]
fn a_new_latch_is_lowered_and_follows_raise_and_lower() {
    let latch = StopLatch::new();
    assert!(!latch.raised());

    latch.raise();
    assert!(latch.raised());

    latch.lower();
    assert!(!latch.raised());
}
