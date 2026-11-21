use amml_runtime::{ItemDrawBook, ItemDrawRefusal};

const OWNER: usize = 1;
const OTHER: usize = 2;

#[test]
fn drop_queues_the_free_to_the_game_thread() {
    let book = ItemDrawBook::new();
    let key = book.add(OWNER, 0x1000).unwrap();
    assert_eq!(book.address(key), Ok(0x1000));

    // The release is allowed on any thread; it only queues, so no address is given to free on this thread.
    std::thread::scope(|scope| scope.spawn(|| book.release(OWNER, key)).join().unwrap()).unwrap();

    assert_eq!(book.address(key), Err(ItemDrawRefusal::Stale));
    assert!(book.has_frees());
    assert_eq!(book.take_frees(), vec![0x1000]);
    assert!(!book.has_frees());
    assert!(book.take_frees().is_empty());
}

#[test]
fn a_second_release_and_a_release_of_another_owner_are_refused() {
    let book = ItemDrawBook::new();
    let key = book.add(OWNER, 0x1000).unwrap();

    assert_eq!(book.release(OTHER, key), Err(ItemDrawRefusal::NotOwner));
    assert_eq!(book.address(key), Ok(0x1000));
    book.release(OWNER, key).unwrap();
    assert_eq!(book.release(OWNER, key), Err(ItemDrawRefusal::Stale));
    assert_eq!(book.take_frees(), vec![0x1000]);
}

#[test]
fn world_leave_frees_every_copy() {
    let book = ItemDrawBook::new();
    let first = book.add(OWNER, 0x1000).unwrap();
    let second = book.add(OTHER, 0x2000).unwrap();
    book.release(OWNER, first).unwrap();

    assert_eq!(book.world_left(), 1);

    let mut frees = book.take_frees();
    frees.sort_unstable();
    assert_eq!(frees, vec![0x1000, 0x2000]);
    assert_eq!(book.address(second), Err(ItemDrawRefusal::Stale));
    assert_eq!(book.release(OTHER, second), Err(ItemDrawRefusal::Stale));
    let newer = book.add(OTHER, 0x3000).unwrap();
    assert!(newer.id > second.id);
    assert_ne!(newer.generation, second.generation);
    assert_eq!(book.address(newer), Ok(0x3000));
}

#[test]
fn the_cap_per_mod_counts_live_copies_only() {
    let book = ItemDrawBook::new();
    let keys: Vec<_> = (0..ItemDrawBook::MAX_PER_OWNER).map(|index| book.add(OWNER, 0x1000 + index).unwrap()).collect();

    assert_eq!(book.add(OWNER, 0x9000), Err(ItemDrawRefusal::TooMany));
    // Another mod has its own count.
    assert!(book.add(OTHER, 0x9000).is_ok());

    book.release(OWNER, keys[0]).unwrap();
    assert!(book.add(OWNER, 0x9001).is_ok());
}

#[test]
fn disabling_a_mod_queues_the_free_of_its_copies_only() {
    let book = ItemDrawBook::new();
    let mine = book.add(OWNER, 0x1000).unwrap();
    let theirs = book.add(OTHER, 0x2000).unwrap();

    assert_eq!(book.disable_owner(OWNER), 1);

    assert_eq!(book.take_frees(), vec![0x1000]);
    assert_eq!(book.address(mine), Err(ItemDrawRefusal::Stale));
    assert_eq!(book.address(theirs), Ok(0x2000));
}
