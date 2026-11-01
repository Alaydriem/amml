use amml_runtime::{ChainError, HookChain, OriginalValue, OriginalWrite};

const TARGET: u64 = 0x1000;

fn last(detour: u64) -> OriginalWrite {
    OriginalWrite { slot: detour + 0x100, value: OriginalValue::Trampoline }
}

fn to(detour: u64, next: u64) -> OriginalWrite {
    OriginalWrite { slot: detour + 0x100, value: OriginalValue::Hook(next) }
}

fn add(chain: &mut HookChain, handle: u64, priority: i32, detour: u64) -> amml_runtime::ChainChange {
    chain.add(TARGET, handle, priority, detour, detour + 0x100).unwrap()
}

#[test]
fn higher_priority_runs_first_and_each_original_points_to_the_next() {
    let mut chain = HookChain::new(8);
    add(&mut chain, 1, 0, 0xB);
    let change_a = add(&mut chain, 2, 10, 0xA);
    let change_c = add(&mut chain, 3, -5, 0xC);

    assert_eq!(change_a.first, Some(0xA));
    assert_eq!(change_a.originals, vec![last(0xB), to(0xA, 0xB)]);
    assert_eq!(change_c.first, Some(0xA));
    // last hook first: the order the engine must write
    assert_eq!(change_c.originals, vec![last(0xC), to(0xB, 0xC), to(0xA, 0xB)]);
}

#[test]
fn equal_priority_keeps_registration_order() {
    let mut chain = HookChain::new(8);
    add(&mut chain, 1, 5, 0xA);
    add(&mut chain, 2, 5, 0xB);
    let change = add(&mut chain, 3, 5, 0xC);

    assert_eq!(change.first, Some(0xA));
    assert_eq!(change.originals, vec![last(0xC), to(0xB, 0xC), to(0xA, 0xB)]);
}

#[test]
fn removing_the_middle_hook_relinks_its_neighbours() {
    let mut chain = HookChain::new(8);
    add(&mut chain, 1, 10, 0xA);
    add(&mut chain, 2, 5, 0xB);
    add(&mut chain, 3, 0, 0xC);

    let change = chain.remove(2).unwrap();

    assert_eq!(change.first, Some(0xA));
    assert_eq!(change.originals, vec![last(0xC), to(0xA, 0xC)]);
}

#[test]
fn removing_the_first_hook_moves_the_target_to_the_second() {
    let mut chain = HookChain::new(8);
    add(&mut chain, 1, 10, 0xA);
    add(&mut chain, 2, 0, 0xB);

    let change = chain.remove(1).unwrap();

    assert_eq!(change.first, Some(0xB));
    assert_eq!(change.originals, vec![last(0xB)]);
}

#[test]
fn removing_the_last_hook_empties_the_target() {
    let mut chain = HookChain::new(8);
    add(&mut chain, 1, 0, 0xA);

    let change = chain.remove(1).unwrap();

    assert_eq!(change.first, None);
    assert!(change.originals.is_empty());
    assert!(chain.is_empty());
}

#[test]
fn targets_have_independent_chains() {
    let mut chain = HookChain::new(8);
    add(&mut chain, 1, 0, 0xA);
    let change = chain.add(TARGET + 0x10, 2, 0, 0xB, 0xB + 0x100).unwrap();

    assert_eq!(change.first, Some(0xB));
    assert_eq!(change.originals, vec![last(0xB)]);
    assert_eq!(chain.target_of(1), Some(TARGET));
    assert_eq!(chain.target_of(2), Some(TARGET + 0x10));
}

#[test]
fn the_hook_limit_holds_and_a_removal_frees_a_place() {
    let mut chain = HookChain::new(2);
    add(&mut chain, 1, 0, 0xA);
    add(&mut chain, 2, 0, 0xB);

    assert_eq!(chain.add(TARGET, 3, 0, 0xC, 0xC + 0x100).unwrap_err(), ChainError::Full(2));
    chain.remove(1).unwrap();
    assert!(chain.add(TARGET, 3, 0, 0xC, 0xC + 0x100).is_ok());
}

#[test]
fn a_reused_or_unknown_handle_is_refused() {
    let mut chain = HookChain::new(8);
    add(&mut chain, 1, 0, 0xA);

    assert_eq!(chain.add(TARGET, 1, 0, 0xB, 0xB + 0x100).unwrap_err(), ChainError::DuplicateHandle(1));
    assert_eq!(chain.remove(9).unwrap_err(), ChainError::UnknownHandle(9));
}

#[test]
fn a_slot_held_by_a_live_hook_is_refused_on_any_target() {
    let mut chain = HookChain::new(8);
    add(&mut chain, 1, 0, 0xA);

    let same_target = chain.add(TARGET, 2, 0, 0xB, 0xA + 0x100).unwrap_err();
    let other_target = chain.add(TARGET + 0x10, 3, 0, 0xC, 0xA + 0x100).unwrap_err();

    assert_eq!(same_target, ChainError::DuplicateSlot(0xA + 0x100));
    assert_eq!(other_target, ChainError::DuplicateSlot(0xA + 0x100));
    assert_eq!(chain.len(), 1);
    chain.remove(1).unwrap();
    assert!(chain.add(TARGET, 2, 0, 0xB, 0xA + 0x100).is_ok());
}

#[test]
fn the_same_detour_twice_on_one_target_is_refused_but_not_across_targets() {
    let mut chain = HookChain::new(8);
    add(&mut chain, 1, 0, 0xA);

    let again = chain.add(TARGET, 2, 5, 0xA, 0x999).unwrap_err();

    assert_eq!(again, ChainError::DuplicateDetour(0xA));
    assert!(chain.add(TARGET + 0x10, 3, 5, 0xA, 0x999).is_ok());
}
