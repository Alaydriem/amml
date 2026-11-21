use amml::EnttLookup;

use crate::support::{FakeRegistry, FakeStorage};

const COMPONENT_SIZE: usize = 0x128;
const CAMERA_TYPE: u32 = 0x4F60_47C7;
const ACTIVE_TYPE: u32 = 0x1130_40A1;

fn image(sparse_pages: &[bool], payload_pages: &[bool]) -> FakeStorage {
    FakeStorage::new(sparse_pages, payload_pages, COMPONENT_SIZE)
}

#[test]
fn page_and_index_split_matches_the_constants() {
    // Entity ids as the game stores them: the low 18 bits are the index, the rest the version.
    assert_eq!(EnttLookup::sparse_position(0x0001_0804), (0x21, 0x004));
    assert_eq!(EnttLookup::sparse_position(0x0040_0003), (0, 3));
    assert_eq!(EnttLookup::sparse_position(0x0003_FFFF), (0x7F, 0x7FF));
    assert_eq!(EnttLookup::payload_position(0x105), (2, 5));
    assert_eq!(EnttLookup::payload_position(0x7F), (0, 0x7F));
    assert_eq!(EnttLookup::payload_position(0x80), (1, 0));
}

#[test]
fn a_present_entity_gives_its_component_address() {
    let mut cameras = image(&[true], &[true, true]);
    cameras.insert(0x0040_0003, 0x81);
    cameras.component_mut(0x81)[0] = 7;
    let registry = FakeRegistry::new(&[(CAMERA_TYPE, &cameras)], 4);

    // SAFETY: the registry and storage images are live and laid out as the game's.
    let found = unsafe {
        let storage = EnttLookup::storage(registry.address(), CAMERA_TYPE).unwrap();
        EnttLookup::component(storage, 0x0040_0003, COMPONENT_SIZE)
    }
    .unwrap();

    // Packed index 0x81 is payload page 1, slot 1.
    assert_eq!(found, cameras.payload_pointers[1] as usize + COMPONENT_SIZE);
    // SAFETY: the address is inside the live payload page.
    assert_eq!(unsafe { std::ptr::with_exposed_provenance::<u8>(found).read() }, 7);
}

#[test]
fn an_entity_with_another_version_or_index_is_not_found() {
    let mut cameras = image(&[true], &[true]);
    cameras.insert(0x0040_0003, 0x01);
    let registry = FakeRegistry::new(&[(CAMERA_TYPE, &cameras)], 4);

    // SAFETY: the registry and storage images are live and laid out as the game's.
    let found = unsafe {
        let storage = EnttLookup::storage(registry.address(), CAMERA_TYPE).unwrap();
        (
            EnttLookup::packed_index(storage, 0x0040_0003),
            EnttLookup::packed_index(storage, 0x0080_0003),
            EnttLookup::packed_index(storage, 0x0040_0004),
        )
    };

    assert_eq!(found, (Some(1), None, None));
}

#[test]
fn a_sparse_page_outside_the_pool_gives_none() {
    let cameras = image(&[true], &[true]);
    let registry = FakeRegistry::new(&[(CAMERA_TYPE, &cameras)], 4);

    // SAFETY: the registry and storage images are live and laid out as the game's.
    let found = unsafe {
        let storage = EnttLookup::storage(registry.address(), CAMERA_TYPE).unwrap();
        // Index 0x1_0804 is sparse page 0x21 and index 0x3_FFFF page 0x7F; the pool has one page.
        (EnttLookup::packed_index(storage, 0x0001_0804), EnttLookup::component(storage, 0x0003_FFFF, COMPONENT_SIZE))
    };

    assert_eq!(found, (None, None));
}

#[test]
fn a_null_sparse_page_gives_none() {
    let cameras = image(&[false, true], &[true]);
    let registry = FakeRegistry::new(&[(CAMERA_TYPE, &cameras)], 4);

    // SAFETY: the registry and storage images are live and laid out as the game's.
    let found = unsafe {
        let storage = EnttLookup::storage(registry.address(), CAMERA_TYPE).unwrap();
        EnttLookup::packed_index(storage, 3)
    };

    assert_eq!(found, None);
}

#[test]
fn a_payload_page_outside_the_pool_or_null_gives_none() {
    let mut cameras = image(&[true], &[true, false]);
    // Packed index 0x80 is payload page 1 (null); 0x100 is page 2 (past the pool).
    cameras.insert(1, 0x80);
    cameras.insert(2, 0x100);
    let registry = FakeRegistry::new(&[(CAMERA_TYPE, &cameras)], 4);

    // SAFETY: the registry and storage images are live and laid out as the game's.
    let found = unsafe {
        let storage = EnttLookup::storage(registry.address(), CAMERA_TYPE).unwrap();
        (EnttLookup::component(storage, 1, COMPONENT_SIZE), EnttLookup::component(storage, 2, COMPONENT_SIZE))
    };

    assert_eq!(found, (None, None));
}

#[test]
fn a_registry_without_the_pool_or_with_unusable_buckets_gives_none() {
    let cameras = image(&[true], &[true]);
    let registry = FakeRegistry::new(&[(CAMERA_TYPE, &cameras)], 4);
    let empty = FakeRegistry::new(&[], 1);
    let mut no_buckets = FakeRegistry::new(&[], 1);
    no_buckets.image[0x50 / 8] = no_buckets.image[0x48 / 8];
    // Three buckets is not a power of two: the game's mask would reach past the vector.
    let three = FakeRegistry::new(&[(CAMERA_TYPE, &cameras)], 3);

    // SAFETY: the registry images are live and laid out as the game's; a null registry is allowed.
    unsafe {
        assert!(EnttLookup::storage(registry.address(), ACTIVE_TYPE).is_none());
        assert!(EnttLookup::storage(empty.address(), CAMERA_TYPE).is_none());
        assert!(EnttLookup::storage(no_buckets.address(), CAMERA_TYPE).is_none());
        assert!(EnttLookup::storage(three.address(), CAMERA_TYPE).is_none());
        assert!(EnttLookup::storage(std::ptr::null(), CAMERA_TYPE).is_none());
    }
}

#[test]
fn a_chain_that_loops_ends_at_none() {
    let cameras = image(&[true], &[true]);
    let mut registry = FakeRegistry::new(&[(CAMERA_TYPE, &cameras)], 1);
    // The node's next index points at itself and its key differs from the one asked for.
    registry.nodes[0][0] = 0;
    registry.point();

    // SAFETY: the registry image is live and laid out as the game's.
    let found = unsafe { EnttLookup::storage(registry.address(), ACTIVE_TYPE) };

    assert!(found.is_none());
}

#[test]
fn a_pool_without_a_storage_gives_none() {
    let cameras = image(&[true], &[true]);
    let mut registry = FakeRegistry::new(&[(CAMERA_TYPE, &cameras)], 4);
    registry.nodes[0][2] = 0;
    registry.point();

    // SAFETY: the registry image is live and laid out as the game's.
    let found = unsafe { EnttLookup::storage(registry.address(), CAMERA_TYPE) };

    assert!(found.is_none());
}
