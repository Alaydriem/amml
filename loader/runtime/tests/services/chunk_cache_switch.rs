use amml_runtime::ChunkCacheSwitch;

#[test]
fn any_false_wish_disables_the_cache() {
    let mut switch = ChunkCacheSwitch::new();
    assert!(!switch.disabled());

    switch.set(1, true);
    assert!(!switch.disabled());

    switch.set(2, false);
    assert!(switch.disabled());

    // A true wish of another mod does not undo a false one.
    switch.set(3, true);
    assert!(switch.disabled());

    // The same mod changing its wish to true lifts the disable.
    switch.set(2, true);
    assert!(!switch.disabled());
}

#[test]
fn disable_clears_the_wish() {
    let mut switch = ChunkCacheSwitch::new();
    switch.set(4, false);
    switch.set(5, false);
    assert!(switch.disabled());

    // Mod 4 is disabled: mod 5 still holds its wish.
    assert!(switch.clear(4));
    assert!(switch.disabled());

    assert!(switch.clear(5));
    assert!(!switch.disabled());

    // A mod with no wish changes nothing.
    assert!(!switch.clear(6));
}
