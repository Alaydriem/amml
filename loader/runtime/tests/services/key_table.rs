use amml_abi::AmmlModifiers;
use amml_runtime::{KeyTable, ResourceId};

const A: ResourceId = ResourceId(1);
const B: ResourceId = ResourceId(2);
const C: ResourceId = ResourceId(3);

#[test]
fn a_second_down_is_a_repeat() {
    let table = KeyTable::new();
    let first = table.real(0x57, true);
    assert!(first.raise && !first.repeat);
    let second = table.real(0x57, true);
    assert!(second.raise && second.repeat);
}

#[test]
fn an_up_of_a_key_that_is_not_down_is_not_raised() {
    let table = KeyTable::new();
    assert!(!table.real(0x57, false).raise);
    assert!(table.finish(0x57, false, false));
}

#[test]
fn keys_from_0x100_are_ignored() {
    let table = KeyTable::new();
    assert!(!table.real(0x100, true).raise);
    assert!(!table.seen_any());
}

#[test]
fn a_cancelled_down_makes_its_up_cancelled_and_others_pass() {
    let table = KeyTable::new();
    table.real(0x41, true);
    assert!(!table.finish(0x41, true, true));
    assert!(table.real(0x41, false).raise);
    assert!(!table.finish(0x41, false, false));

    table.real(0x42, true);
    assert!(table.finish(0x42, true, false));
    table.real(0x42, false);
    assert!(table.finish(0x42, false, true));
}

#[test]
fn left_and_generic_modifiers_set_their_own_bits() {
    let table = KeyTable::new();
    table.real(0xA0, true);
    assert_eq!(table.modifiers(), AmmlModifiers::LSHIFT);
    table.real(0x10, true);
    assert_eq!(table.modifiers().0, AmmlModifiers::LSHIFT.0 | AmmlModifiers::SHIFT.0);
    table.real(0xA0, false);
    assert_eq!(table.modifiers(), AmmlModifiers::SHIFT);
}

#[test]
fn a_cancelled_shift_down_still_counts_as_down() {
    let table = KeyTable::new();
    table.real(0xA0, true);
    assert!(!table.finish(0xA0, true, true));
    assert_eq!(table.is_down(0xA0), Some(true));
    assert_eq!(table.modifiers(), AmmlModifiers::LSHIFT);
}

#[test]
fn is_down_is_none_before_the_first_event() {
    let table = KeyTable::new();
    assert_eq!(table.is_down(0x57), None);
    table.real(0x41, true);
    assert_eq!(table.is_down(0x57), Some(false));
}

#[test]
fn a_second_injected_holder_keeps_the_key_down_until_the_last_up() {
    let table = KeyTable::new();
    assert!(table.injected(A, 0x57, true));
    assert!(!table.injected(B, 0x57, true));
    assert!(!table.injected(B, 0x57, false));
    assert!(table.injected(A, 0x57, false));
    assert!(!table.injected(A, 0x57, false));
}

#[test]
fn an_injected_hold_of_a_real_held_key_sends_neither_down_nor_up() {
    let table = KeyTable::new();
    table.real(0x57, true);
    assert!(!table.injected(A, 0x57, true));
    assert!(!table.injected(A, 0x57, false));
    assert_eq!(table.is_down(0x57), Some(true));
}

#[test]
fn a_real_up_during_an_injected_hold_ends_it() {
    let table = KeyTable::new();
    table.real(0x57, true);
    assert!(!table.injected(A, 0x57, true));
    assert!(table.real(0x57, false).raise);
    assert!(!table.injected(A, 0x57, false));
    assert!(table.injected(A, 0x57, true));
}

#[test]
fn an_injected_up_after_a_real_reset_up_is_dropped() {
    let table = KeyTable::new();
    assert!(table.injected(A, 0x57, true));
    table.real(0x57, false);
    assert!(!table.injected(A, 0x57, false));
}

#[test]
fn a_cancelled_repeat_does_not_swallow_the_up() {
    let table = KeyTable::new();
    table.real(0x57, true);
    assert!(table.finish(0x57, true, false));
    table.real(0x57, true);
    assert!(!table.finish(0x57, true, true));
    table.real(0x57, false);
    assert!(table.finish(0x57, false, false));
}

#[test]
fn a_repeat_of_a_cancelled_down_stays_cancelled() {
    let table = KeyTable::new();
    table.real(0x57, true);
    assert!(!table.finish(0x57, true, true));
    table.real(0x57, true);
    assert!(!table.finish(0x57, true, false));
    table.real(0x57, false);
    assert!(!table.finish(0x57, false, false));
}

#[test]
fn an_injected_shift_is_not_in_modifiers_or_is_down() {
    let table = KeyTable::new();
    assert!(table.injected(A, 0xA0, true));
    assert_eq!(table.modifiers(), AmmlModifiers(0));
    assert_eq!(table.is_down(0xA0), Some(false));
    assert!(!table.real(0xA0, true).repeat);
}

#[test]
fn an_up_from_a_holder_before_a_real_up_does_not_release_a_newer_holder() {
    let table = KeyTable::new();
    assert!(table.injected(A, 0x57, true));
    table.real(0x57, true);
    assert!(table.real(0x57, false).raise);
    assert!(table.injected(C, 0x57, true));
    assert!(!table.injected(A, 0x57, false));
    assert!(table.injected(C, 0x57, false));
}

#[test]
fn two_holders_before_a_real_up_leave_a_newer_holder_alone() {
    let table = KeyTable::new();
    assert!(table.injected(A, 0x57, true));
    assert!(!table.injected(B, 0x57, true));
    table.real(0x57, false);
    assert!(table.injected(C, 0x57, true));
    assert!(!table.injected(A, 0x57, false));
    assert!(!table.injected(B, 0x57, false));
    assert!(table.injected(C, 0x57, false));
}

#[test]
fn a_second_down_from_the_same_lease_is_dropped() {
    let table = KeyTable::new();
    assert!(table.injected(A, 0x57, true));
    assert!(!table.injected(A, 0x57, true));
    assert!(table.injected(A, 0x57, false));
}

#[test]
fn an_injected_up_reaches_the_game_while_a_blocked_real_press_is_held() {
    let table = KeyTable::new();
    assert!(table.injected(A, 0x57, true));
    table.real(0x57, true);
    assert!(!table.finish(0x57, true, true));
    assert!(table.injected(A, 0x57, false));
    assert!(table.real(0x57, false).raise);
    assert!(!table.finish(0x57, false, false));
}

#[test]
fn an_injected_down_reaches_the_game_while_a_blocked_real_press_is_held() {
    let table = KeyTable::new();
    table.real(0x57, true);
    assert!(!table.finish(0x57, true, true));
    assert!(table.injected(A, 0x57, true));
    table.real(0x57, false);
    assert!(!table.finish(0x57, false, false));
    assert!(table.holds(A, 0x57));
    assert!(table.injected(A, 0x57, false));
}

#[test]
fn a_real_up_that_reached_the_game_ends_the_holders() {
    let table = KeyTable::new();
    assert!(table.injected(A, 0x57, true));
    assert!(table.holds(A, 0x57));
    table.real(0x57, true);
    assert!(table.finish(0x57, true, false));
    table.real(0x57, false);
    assert!(!table.holds(A, 0x57));
}

#[test]
fn a_real_up_gives_the_leases_whose_hold_it_ended() {
    let table = KeyTable::new();
    assert!(table.injected(A, 0x57, true));
    assert!(!table.injected(B, 0x57, true));
    assert_eq!(table.real(0x57, false).ended, vec![A, B]);
    assert!(table.real(0x57, false).ended.is_empty());
}

#[test]
fn the_up_of_a_held_back_press_ends_no_hold() {
    let table = KeyTable::new();
    table.real(0x57, true);
    assert!(!table.finish(0x57, true, true));
    assert!(table.injected(A, 0x57, true));
    assert!(table.real(0x57, false).ended.is_empty());
    assert!(table.holds(A, 0x57));
}

#[test]
fn real_down_keys_lists_the_keys_the_real_feed_holds_in_key_order() {
    let table = KeyTable::new();
    assert!(table.real_down_keys().is_empty());
    table.real(0x57, true);
    table.real(0x12, true);
    table.real(0x41, true);
    table.real(0x41, false);

    assert_eq!(table.real_down_keys(), vec![0x12, 0x57]);
}

#[test]
fn real_down_keys_leaves_out_keys_held_only_through_injection() {
    let table = KeyTable::new();
    table.injected(A, 0x57, true);
    table.real(0x11, true);
    table.finish(0x11, true, true);

    assert_eq!(table.real_down_keys(), vec![0x11]);
}
