use core::ffi::c_void;

use amml_abi::{AmmlActorTable, AmmlBlockPos, AmmlFace, AmmlHitResult, AmmlRotation, AmmlStatus, AmmlStr, AmmlVec3};

unsafe extern "C" fn is_alive(_: *const c_void, _: *mut u8) -> AmmlStatus {
    AmmlStatus::OK
}
unsafe extern "C" fn health(_: *const c_void, _: *mut i32) -> AmmlStatus {
    AmmlStatus::OK
}

unsafe extern "C" fn set_rotation(_: *const c_void, _: *const AmmlRotation, _: f32, _: u8) -> AmmlStatus {
    AmmlStatus::OK
}

unsafe extern "C" fn build_at(
    _: AmmlStr,
    _: *const c_void,
    _: *const AmmlBlockPos,
    _: AmmlFace,
    _: *const AmmlVec3,
    _: *mut u8,
) -> AmmlStatus {
    AmmlStatus::OK
}

unsafe extern "C" fn hit_result(_: *const c_void, _: *mut AmmlHitResult) -> AmmlStatus {
    AmmlStatus::OK
}

fn table() -> AmmlActorTable {
    AmmlActorTable {
        size: AmmlActorTable::SIZE_V6,
        version: 6,
        is_alive,
        position: None,
        eye_position: None,
        rotation: None,
        dimension: None,
        dimension_id: None,
        head_position: None,
        block_source: None,
        health: Some(health),
        view_vector: None,
        aabb: None,
        game_type: None,
        permission_level: None,
        set_rotation: Some(set_rotation),
        build_at: Some(build_at),
        build_delay: None,
        max_pick_range: None,
        hit_result: Some(hit_result),
    }
}

#[test]
fn actor_table_v2_reads_none_for_v3_fields() {
    let mut table = table();
    table.size = AmmlActorTable::SIZE_V2;
    // SAFETY: the table lives for the call; the read covers only its first `size` bytes (the v2 size).
    let read = unsafe { AmmlActorTable::read(&raw const table) }.unwrap();
    assert!(read.health.is_none());
    assert!(read.head_position.is_none());
    assert!(read.block_source.is_none());
}

#[test]
fn actor_table_v3_reads_none_for_v4_fields() {
    let mut table = table();
    table.size = AmmlActorTable::SIZE_V3;
    // SAFETY: the table lives for the call; the read covers only its first `size` bytes (the v3 size).
    let read = unsafe { AmmlActorTable::read(&raw const table) }.unwrap();
    assert!(read.health.is_some());
    assert!(read.view_vector.is_none());
    assert!(read.aabb.is_none());
    assert!(read.game_type.is_none());
    assert!(read.permission_level.is_none());
    assert!(read.set_rotation.is_none());
}

#[test]
fn actor_table_v4_reads_none_for_v5_fields() {
    let mut table = table();
    table.size = AmmlActorTable::SIZE_V4;
    // SAFETY: the table lives for the call; the read covers only its first `size` bytes (the v4 size).
    let read = unsafe { AmmlActorTable::read(&raw const table) }.unwrap();
    assert!(read.set_rotation.is_some());
    assert!(read.build_at.is_none());
}

#[test]
fn actor_table_v5_reads_its_build_function() {
    let table = table();
    // SAFETY: the table lives for the call.
    let read = unsafe { AmmlActorTable::read(&raw const table) }.unwrap();
    assert!(read.build_at.is_some());
}

#[test]
fn actor_table_v4_reads_its_rotation_writer() {
    let table = table();
    // SAFETY: the table lives for the call.
    let read = unsafe { AmmlActorTable::read(&raw const table) }.unwrap();
    assert!(read.set_rotation.is_some());
}

#[test]
fn actor_table_v3_reads_its_health_function() {
    let table = table();
    // SAFETY: the table lives for the call.
    let read = unsafe { AmmlActorTable::read(&raw const table) }.unwrap();
    assert!(read.health.is_some());
}

#[test]
fn an_actor_table_shorter_than_v1_does_not_read() {
    let mut table = table();
    table.size = AmmlActorTable::SIZE_V1 - 8;
    // SAFETY: the table lives for the call.
    assert!(unsafe { AmmlActorTable::read(&raw const table) }.is_none());
}

#[test]
fn actor_table_v5_reads_none_for_hit_result_and_v6_reads_it() {
    let mut old = table();
    old.size = AmmlActorTable::SIZE_V5;
    let new = table();
    // SAFETY: the tables live for the calls; each read covers only its first `size` bytes.
    let (old, new) = unsafe { (AmmlActorTable::read(&raw const old), AmmlActorTable::read(&raw const new)) };
    assert!(old.unwrap().hit_result.is_none());
    assert!(new.unwrap().hit_result.is_some());
    assert_eq!(AmmlActorTable::SIZE_V6, 152);
}
