use amml_runtime::{GroupBook, GroupRefusal};

const OWNER: usize = 1;
const OTHER: usize = 2;

#[test]
fn group_is_stale_after_world_left() {
    let book = GroupBook::new();
    let group = book.open(OWNER, 0x100).unwrap();
    assert_eq!(book.handle(OWNER, group), Ok(0x100));

    assert_eq!(book.world_left(), 1);

    assert_eq!(book.handle(OWNER, group), Err(GroupRefusal::Stale));
    assert_eq!(book.finish(OWNER, group), Err(GroupRefusal::Stale));
    // A group of the next world never takes the id or the generation of the stale one.
    let next = book.open(OWNER, 0x200).unwrap();
    assert_ne!(next, group);
    assert_eq!(book.handle(OWNER, group), Err(GroupRefusal::Stale));
    // The game buffer waits for the render thread, once.
    assert!(book.has_due());
    assert_eq!(book.take_due(), vec![0x100]);
    assert!(!book.has_due());
    assert!(book.take_due().is_empty());
}

#[test]
fn group_is_stale_after_device_reset_and_disable() {
    let book = GroupBook::new();
    let reset = book.open(OWNER, 0x100).unwrap();
    assert_eq!(book.device_reset(), 1);
    assert_eq!(book.handle(OWNER, reset), Err(GroupRefusal::Stale));

    let mine = book.open(OWNER, 0x200).unwrap();
    let theirs = book.open(OTHER, 0x300).unwrap();
    assert_eq!(book.disable_owner(OWNER), 1);

    assert_eq!(book.handle(OWNER, mine), Err(GroupRefusal::Stale));
    assert_eq!(book.handle(OTHER, theirs), Ok(0x300));
    let mut due = book.take_due();
    due.sort_unstable();
    assert_eq!(due, vec![0x100, 0x200]);
}

#[test]
fn group_cap_is_64_per_mod() {
    let book = GroupBook::new();
    let groups: Vec<_> = (0..GroupBook::MAX_PER_MOD).map(|index| book.open(OWNER, 0x1000 + index).unwrap()).collect();
    assert_eq!(GroupBook::MAX_PER_MOD, 64);

    assert_eq!(book.open(OWNER, 0x9000), Err(GroupRefusal::TooMany));
    // The cap is for each mod.
    assert!(book.open(OTHER, 0x9001).is_ok());
    // A finished group gives its place back.
    book.finish(OWNER, groups[0]).unwrap();
    assert!(book.open(OWNER, 0x9002).is_ok());
}

#[test]
fn finish_consumes_the_group() {
    let book = GroupBook::new();
    let group = book.open(OWNER, 0x100).unwrap();

    assert_eq!(book.finish(OWNER, group), Ok(0x100));

    assert_eq!(book.handle(OWNER, group), Err(GroupRefusal::Stale));
    assert_eq!(book.finish(OWNER, group), Err(GroupRefusal::Stale));
    // The caller owns the buffer after the finish: nothing is queued for the render thread.
    assert!(!book.has_due());
    assert_eq!(book.len(), 0);
}

#[test]
fn a_group_belongs_to_its_mod() {
    let book = GroupBook::new();
    let group = book.open(OWNER, 0x100).unwrap();

    assert_eq!(book.handle(OTHER, group), Err(GroupRefusal::NotOwner));
    assert_eq!(book.finish(OTHER, group), Err(GroupRefusal::NotOwner));
    assert_eq!(book.handle(OWNER, group), Ok(0x100));
}

#[test]
fn a_discarded_group_frees_its_slot_and_queues_its_buffer_once() {
    let book = GroupBook::new();
    // More discards than a mod may hold groups: each one gives the slot back.
    for round in 0..(GroupBook::MAX_PER_MOD + 1) {
        let group = book.open(OWNER, 0x1000 + round).unwrap();
        assert_eq!(book.discard(OWNER, group), Ok(()));
        assert_eq!(book.handle(OWNER, group), Err(GroupRefusal::Stale));
        assert_eq!(book.discard(OWNER, group), Err(GroupRefusal::Stale));
    }
    let theirs = book.open(OTHER, 0x300).unwrap();
    assert_eq!(book.discard(OWNER, theirs), Err(GroupRefusal::NotOwner));

    assert!(book.has_due());
    assert_eq!(book.take_due(), (0..(GroupBook::MAX_PER_MOD + 1)).map(|round| 0x1000 + round).collect::<Vec<_>>());
    assert_eq!(book.handle(OTHER, theirs), Ok(0x300));
}
