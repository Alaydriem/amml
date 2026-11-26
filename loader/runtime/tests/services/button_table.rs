use amml_abi::AmmlMouseButton;
use amml_runtime::{ButtonTable, ResourceId};

const A: ResourceId = ResourceId(1);
const B: ResourceId = ResourceId(2);
const C: ResourceId = ResourceId(3);

#[test]
fn a_second_injected_holder_keeps_the_button_down_until_the_last_up() {
    let table = ButtonTable::new();
    assert!(table.injected(A, AmmlMouseButton::LEFT, true));
    assert!(!table.injected(B, AmmlMouseButton::LEFT, true));
    assert!(!table.injected(B, AmmlMouseButton::LEFT, false));
    assert!(table.injected(A, AmmlMouseButton::LEFT, false));
    assert!(!table.injected(A, AmmlMouseButton::LEFT, false));
}

#[test]
fn an_injected_hold_of_a_real_held_button_sends_neither_down_nor_up() {
    let table = ButtonTable::new();
    table.real(AmmlMouseButton::RIGHT, true);
    assert!(!table.injected(A, AmmlMouseButton::RIGHT, true));
    assert!(!table.injected(A, AmmlMouseButton::RIGHT, false));
    assert_eq!(table.is_down(AmmlMouseButton::RIGHT), Some(true));
}

#[test]
fn a_real_up_during_an_injected_hold_ends_it() {
    let table = ButtonTable::new();
    assert!(table.injected(A, AmmlMouseButton::X1, true));
    table.real(AmmlMouseButton::X1, false);
    assert!(!table.injected(A, AmmlMouseButton::X1, false));
}

#[test]
fn an_unknown_button_is_dropped() {
    let table = ButtonTable::new();
    assert!(!table.injected(A, AmmlMouseButton(9), true));
    assert_eq!(table.is_down(AmmlMouseButton(9)), None);
}

#[test]
fn an_up_from_a_holder_before_a_real_up_does_not_release_a_newer_holder() {
    let table = ButtonTable::new();
    assert!(table.injected(A, AmmlMouseButton::LEFT, true));
    table.real(AmmlMouseButton::LEFT, true);
    table.real(AmmlMouseButton::LEFT, false);
    assert!(table.injected(C, AmmlMouseButton::LEFT, true));
    assert!(!table.injected(A, AmmlMouseButton::LEFT, false));
    assert!(table.injected(C, AmmlMouseButton::LEFT, false));
}

#[test]
fn holds_follows_the_recorded_holders() {
    let table = ButtonTable::new();
    assert!(!table.holds(A, AmmlMouseButton::LEFT));
    table.injected(A, AmmlMouseButton::LEFT, true);
    assert!(table.holds(A, AmmlMouseButton::LEFT) && !table.holds(B, AmmlMouseButton::LEFT));
    table.real(AmmlMouseButton::LEFT, false);
    assert!(!table.holds(A, AmmlMouseButton::LEFT));
}

#[test]
fn a_real_up_gives_the_leases_whose_hold_it_ended() {
    let table = ButtonTable::new();
    table.injected(A, AmmlMouseButton::LEFT, true);
    table.injected(B, AmmlMouseButton::LEFT, true);
    assert_eq!(table.real(AmmlMouseButton::LEFT, false), vec![A, B]);
    assert!(table.real(AmmlMouseButton::LEFT, false).is_empty());
    assert!(table.real(AmmlMouseButton::LEFT, true).is_empty());
}
