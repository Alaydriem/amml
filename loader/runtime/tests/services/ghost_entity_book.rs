use amml_runtime::{GhostEntity, GhostEntityBook, GhostEntityRefusal};

const OWNER: usize = 1;
const OTHER: usize = 2;

// A ghost whose game actor is at `actor` with the control block `actor + 0x40`, of the block 0x500, with no skull pose.
fn ghost(actor: usize) -> GhostEntity {
    GhostEntity { actor, control: actor + 0x40, block: 0x500, skull: None, chest_paired: false }
}

#[test]
fn ghost_entity_is_stale_after_disable() {
    let book = GhostEntityBook::new();
    let mine = book.create(OWNER, ghost(0x100)).unwrap();
    let theirs = book.create(OTHER, ghost(0x200)).unwrap();
    assert_eq!(book.entry(OWNER, mine), Ok(ghost(0x100)));

    assert_eq!(book.disable_owner(OWNER), 1);

    assert_eq!(book.entry(OWNER, mine), Err(GhostEntityRefusal::Stale));
    assert_eq!(book.free(OWNER, mine), Err(GhostEntityRefusal::Stale));
    assert_eq!(book.entry(OTHER, theirs), Ok(ghost(0x200)));
    // The game actor waits for the render thread, once.
    assert!(book.has_due());
    assert_eq!(book.take_due(), vec![ghost(0x100)]);
    assert!(!book.has_due());
    assert!(book.take_due().is_empty());
}

#[test]
fn pool_cap_is_256_per_mod() {
    let book = GhostEntityBook::new();
    let ghosts: Vec<_> = (0..GhostEntityBook::MAX_PER_MOD)
        .map(|index| book.create(OWNER, ghost(0x1000 + index * 0x80)).unwrap())
        .collect();
    assert_eq!(GhostEntityBook::MAX_PER_MOD, 256);

    assert!(!book.has_room(OWNER));
    assert_eq!(book.create(OWNER, ghost(0x90000)), Err(GhostEntityRefusal::TooMany));
    // The cap is for each mod.
    assert!(book.has_room(OTHER));
    assert!(book.create(OTHER, ghost(0x90100)).is_ok());
    // A freed ghost gives its place back.
    book.free(OWNER, ghosts[0]).unwrap();
    assert!(book.has_room(OWNER));
    assert!(book.create(OWNER, ghost(0x90200)).is_ok());
}

#[test]
fn ghost_entity_is_stale_after_world_left_and_device_reset() {
    let book = GhostEntityBook::new();
    let left = book.create(OWNER, ghost(0x100)).unwrap();
    assert_eq!(book.world_left(), 1);
    assert_eq!(book.entry(OWNER, left), Err(GhostEntityRefusal::Stale));
    // A ghost of the next world never takes the id or the generation of the stale one.
    let next = book.create(OWNER, ghost(0x200)).unwrap();
    assert_ne!(next, left);
    assert_eq!(book.entry(OWNER, left), Err(GhostEntityRefusal::Stale));

    assert_eq!(book.device_reset(), 1);

    assert_eq!(book.entry(OWNER, next), Err(GhostEntityRefusal::Stale));
    assert_eq!(book.take_due(), vec![ghost(0x100), ghost(0x200)]);
    assert!(book.is_empty());
}

#[test]
fn free_queues_the_actor_once_and_a_ghost_belongs_to_its_mod() {
    let book = GhostEntityBook::new();
    let made = book.create(OWNER, ghost(0x100)).unwrap();
    assert_eq!(book.entry(OTHER, made), Err(GhostEntityRefusal::NotOwner));
    assert_eq!(book.free(OTHER, made), Err(GhostEntityRefusal::NotOwner));
    assert!(!book.has_due());

    book.free(OWNER, made).unwrap();

    assert_eq!(book.free(OWNER, made), Err(GhostEntityRefusal::Stale));
    assert_eq!(book.entry(OWNER, made), Err(GhostEntityRefusal::Stale));
    assert_eq!(book.take_due(), vec![ghost(0x100)]);
    // A later world leave finds nothing left to queue.
    assert_eq!(book.world_left(), 0);
    assert!(book.take_due().is_empty());
}
