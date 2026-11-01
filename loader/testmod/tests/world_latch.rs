use amml_testmod::WorldLatch;

#[test]
fn only_a_change_of_the_world_state_is_reported() {
    let latch = WorldLatch::new();
    let reports: Vec<bool> =
        [false, false, true, true, false, false, true].iter().map(|state| latch.set(*state)).collect();
    assert_eq!(reports, vec![true, false, true, false, true, false, true]);
}
