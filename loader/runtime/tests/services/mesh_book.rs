use amml_runtime::{MeshBook, MeshFree, MeshKey, MeshRefusal};

const OWNER: usize = 1;
const OTHER: usize = 2;

#[test]
fn ids_are_never_reused() {
    let book = MeshBook::new();
    let first = book.create(OWNER, 0x100);
    book.free(OWNER, first).unwrap();

    let second = book.create(OWNER, 0x200);
    book.world_left();
    let third = book.create(OWNER, 0x300);

    assert!(first.id < second.id && second.id < third.id);
    assert_ne!(second.generation, third.generation);
}

#[test]
fn ids_go_stale_at_world_leave_device_reset_and_graphics_switch() {
    let book = MeshBook::new();

    let before_leave = book.create(OWNER, 0x100);
    assert_eq!(book.world_left(), 1);
    assert_eq!(book.handle(OWNER, before_leave), Err(MeshRefusal::Stale));
    assert_eq!(book.free(OWNER, before_leave), Err(MeshRefusal::Stale));

    let before_reset = book.create(OWNER, 0x200);
    assert_eq!(book.device_reset(), 1);
    assert_eq!(book.handle(OWNER, before_reset), Err(MeshRefusal::Stale));

    // The first graphics mode only records it; a change of the mode frees the meshes.
    assert!(!book.note_vibrant(false));
    let before_switch = book.create(OWNER, 0x300);
    assert!(!book.note_vibrant(false));
    assert!(book.is_current(before_switch));
    assert!(book.note_vibrant(true));
    assert_eq!(book.handle(OWNER, before_switch), Err(MeshRefusal::Stale));

    // The game objects of all three are given to the render thread, once.
    let due: Vec<usize> = book.take_due().iter().map(|free| free.handle).collect();
    assert_eq!(due, vec![0x100, 0x200, 0x300]);
    assert!(book.take_due().is_empty());

    let after = book.create(OWNER, 0x400);
    assert_eq!(book.handle(OWNER, after), Ok(0x400));
}

#[test]
fn free_off_thread_is_queued_to_the_render_call() {
    let book = MeshBook::new();
    let key = book.create(OWNER, 0x100);
    assert!(!book.has_due());

    std::thread::scope(|scope| {
        scope.spawn(|| book.free(OWNER, key).unwrap());
    });

    // The mod sees the mesh gone at once; the game object waits for the render thread.
    assert_eq!(book.handle(OWNER, key), Err(MeshRefusal::Stale));
    assert!(book.has_due());
    assert_eq!(book.take_due(), vec![MeshFree { key, handle: 0x100 }]);
    assert!(!book.has_due());
}

#[test]
fn disable_frees_only_that_mods_meshes() {
    let book = MeshBook::new();
    let mine_a = book.create(OWNER, 0x100);
    let mine_b = book.create(OWNER, 0x200);
    let theirs = book.create(OTHER, 0x300);

    assert_eq!(book.disable_owner(OWNER), 2);

    assert!(!book.is_current(mine_a) && !book.is_current(mine_b));
    assert_eq!(book.handle(OTHER, theirs), Ok(0x300));
    let due: Vec<MeshKey> = book.take_due().iter().map(|free| free.key).collect();
    assert_eq!(due, vec![mine_a, mine_b]);
    assert_eq!(book.len(), 1);
}

#[test]
fn a_mesh_of_another_mod_is_refused() {
    let book = MeshBook::new();
    let key = book.create(OWNER, 0x100);

    assert_eq!(book.free(OTHER, key), Err(MeshRefusal::NotOwner));
    assert_eq!(book.handle(OTHER, key), Err(MeshRefusal::NotOwner));
    assert!(book.is_current(key));
    assert!(!book.has_due());
}

#[test]
fn a_forgotten_mode_is_recorded_again_without_a_switch() {
    let book = MeshBook::new();
    assert!(!book.note_vibrant(true));
    let mesh = book.create(OWNER, 0x100);

    // The reader stopped while the game switched: the next read records the new mode and frees nothing.
    book.forget_vibrant();
    assert!(!book.note_vibrant(false));

    assert!(book.is_current(mesh));
    assert!(book.note_vibrant(true));
    assert!(!book.is_current(mesh));
}
