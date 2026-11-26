use amml_runtime::services::ResourceStore;
use amml_runtime::structs::{ResourceError, ResourceId, ResourceKind, WorldEpoch};

const MESH: ResourceKind = ResourceKind(1);
const TEXTURE: ResourceKind = ResourceKind(2);
const NO_WORLD: WorldEpoch = WorldEpoch(0);

#[test]
fn an_id_from_a_left_world_is_stale() {
    let store = ResourceStore::<u32>::new();
    let id = store.insert(MESH, 1, WorldEpoch(3), 10).unwrap();
    let taken = store.take_epoch(WorldEpoch(3));
    assert_eq!(taken, vec![(MESH, 10)]);
    assert_eq!(store.get(id), Err(ResourceError::Stale));
}

#[test]
fn a_reused_slot_does_not_revive_an_old_id() {
    let store = ResourceStore::<u32>::new();
    let a = store.insert(MESH, 1, NO_WORLD, 1).unwrap();
    assert_eq!(store.remove(a), Ok(1));
    let b = store.insert(MESH, 1, NO_WORLD, 2).unwrap();
    assert_ne!(a, b);
    assert_eq!(store.get(a), Err(ResourceError::Stale));
    assert_eq!(store.get(b), Ok(2));
    assert_eq!(store.remove(a), Err(ResourceError::Stale));
}

#[test]
fn take_owner_returns_only_that_owners_resources() {
    let store = ResourceStore::<u32>::new();
    let first = store.insert(MESH, 1, NO_WORLD, 1).unwrap();
    let second = store.insert(MESH, 2, NO_WORLD, 2).unwrap();
    let third = store.insert(TEXTURE, 1, NO_WORLD, 3).unwrap();
    let mut taken = store.take_owner(1);
    taken.sort_by_key(|(_, value)| *value);
    assert_eq!(taken, vec![(MESH, 1), (TEXTURE, 3)]);
    assert_eq!(store.get(first), Err(ResourceError::Stale));
    assert_eq!(store.get(third), Err(ResourceError::Stale));
    assert_eq!(store.get(second), Ok(2));
}

#[test]
fn an_id_of_another_kind_is_refused() {
    let store = ResourceStore::<u32>::new();
    let id = store.insert(MESH, 1, NO_WORLD, 1).unwrap();
    assert_eq!(store.get_kind(id, TEXTURE), Err(ResourceError::WrongKind));
    assert_eq!(store.get_kind(id, MESH), Ok(1));
}

#[test]
fn world_free_resources_are_not_taken_by_an_epoch() {
    let store = ResourceStore::<u32>::new();
    let id = store.insert(MESH, 1, NO_WORLD, 1).unwrap();
    assert!(store.take_epoch(WorldEpoch(5)).is_empty());
    assert!(store.take_epoch(NO_WORLD).is_empty());
    assert_eq!(store.get(id), Ok(1));
}

#[test]
fn take_kind_returns_every_resource_of_that_kind() {
    let store = ResourceStore::<u32>::new();
    let mesh = store.insert(MESH, 1, NO_WORLD, 1).unwrap();
    let texture_a = store.insert(TEXTURE, 1, NO_WORLD, 2).unwrap();
    let texture_b = store.insert(TEXTURE, 2, WorldEpoch(4), 3).unwrap();
    let mut taken = store.take_kind(TEXTURE);
    taken.sort();
    assert_eq!(taken, vec![2, 3]);
    assert_eq!(store.get(texture_a), Err(ResourceError::Stale));
    assert_eq!(store.get(texture_b), Err(ResourceError::Stale));
    assert_eq!(store.get(mesh), Ok(1));
}

#[test]
fn an_id_with_another_kind_byte_is_stale() {
    let store = ResourceStore::<u32>::new();
    let id = store.insert(MESH, 1, NO_WORLD, 1).unwrap();
    let forged = ResourceId((id.0 & !(0xFF << 56)) | (u64::from(TEXTURE.0) << 56));
    assert_eq!(store.get(forged), Err(ResourceError::Stale));
    assert_eq!(store.remove(forged), Err(ResourceError::Stale));
    assert_eq!(store.get(id), Ok(1));
}

#[test]
fn a_slot_whose_generation_runs_out_is_never_reused() {
    let store = ResourceStore::<u32>::new();
    let first = store.insert(MESH, 1, NO_WORLD, 0).unwrap();
    let slot_of = |id: ResourceId| id.0 & 0xFFFF_FFFF;
    // The generation has 24 bits; cycle one slot until it is retired.
    let mut last = first;
    for _ in 0..(1u32 << 24) {
        store.remove(last).unwrap();
        last = store.insert(MESH, 1, NO_WORLD, 0).unwrap();
        if slot_of(last) != slot_of(first) {
            break;
        }
    }
    assert_ne!(slot_of(last), slot_of(first));
    // The retired slot stays out of reuse, however many ids follow.
    for _ in 0..8 {
        let next = store.insert(MESH, 1, NO_WORLD, 0).unwrap();
        assert_ne!(slot_of(next), slot_of(first));
    }
    assert_eq!(store.get(first), Err(ResourceError::Stale));
    assert_eq!(store.get(last), Ok(0));
}

#[test]
fn an_insert_for_a_world_already_taken_back_is_stale() {
    let store = ResourceStore::<u32>::new();
    assert!(store.take_epoch(WorldEpoch(3)).is_empty());

    assert_eq!(store.insert(MESH, 1, WorldEpoch(3), 7), Err((ResourceError::Stale, 7)));
    assert_eq!(store.insert(MESH, 1, WorldEpoch(2), 8), Err((ResourceError::Stale, 8)));
    let next_world = store.insert(MESH, 1, WorldEpoch(4), 9).unwrap();
    let no_world = store.insert(MESH, 1, NO_WORLD, 10).unwrap();
    assert_eq!(store.get(next_world), Ok(9));
    assert_eq!(store.get(no_world), Ok(10));
}
