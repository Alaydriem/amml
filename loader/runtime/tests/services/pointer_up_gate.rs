use amml_runtime::PointerUpGate;

const VIEW: u64 = 7;

#[test]
fn up_of_a_cancelled_down_is_cancelled() {
    let gate = PointerUpGate::new();
    assert!(!gate.down(VIEW, 0, true));
    assert!(gate.up(VIEW, 0));
    // The hold-back ends with the up it covered.
    assert!(!gate.up(VIEW, 0));
}

#[test]
fn an_up_of_a_down_that_reached_the_game_is_not_held_back() {
    let gate = PointerUpGate::new();
    assert!(gate.down(VIEW, 0, false));
    assert!(!gate.up(VIEW, 0));
    // An up the gate saw no down for reaches the game too.
    assert!(!gate.up(VIEW, 1));
}

#[test]
fn new_down_clears_a_held_back_up() {
    let gate = PointerUpGate::new();
    assert!(!gate.down(VIEW, 0, true));
    // The release was lost (a focus change); the next down of the pointer starts a new press.
    assert!(gate.down(VIEW, 0, false));
    assert!(!gate.up(VIEW, 0));
}

#[test]
fn a_new_cancelled_down_holds_back_its_own_up() {
    let gate = PointerUpGate::new();
    assert!(gate.down(VIEW, 0, false));
    assert!(!gate.down(VIEW, 0, true));
    assert!(gate.up(VIEW, 0));
}

#[test]
fn a_cancel_of_one_pointer_or_view_leaves_the_others_alone() {
    let gate = PointerUpGate::new();
    assert!(!gate.down(VIEW, 0, true));
    assert!(!gate.up(VIEW, 1));
    assert!(!gate.up(VIEW + 1, 0));
    assert!(gate.up(VIEW, 0));
}

#[test]
fn ids_not_in_the_pointer_list_pass_through() {
    let ids = [0x64, 0x65];
    assert_eq!(PointerUpGate::pointer_of(ids, 0x64), Some(0));
    assert_eq!(PointerUpGate::pointer_of(ids, 0x65), Some(1));
    assert_eq!(PointerUpGate::pointer_of(ids, 0x66), None);
    assert_eq!(PointerUpGate::pointer_of(ids, 0), None);
}

#[test]
fn the_book_of_held_back_ups_is_bounded() {
    let gate = PointerUpGate::new();
    for view in 0..10_000u64 {
        assert!(!gate.down(view, 0, true));
    }
    assert!(gate.held() <= PointerUpGate::MAX_HELD);
}

#[test]
fn clear_forgets_every_hold_back() {
    let gate = PointerUpGate::new();
    assert!(!gate.down(VIEW, 0, true));
    gate.clear();
    assert_eq!(gate.held(), 0);
    assert!(!gate.up(VIEW, 0));
}
