use core::ffi::c_void;

use amml_abi::{AmmlActorTable, AmmlStatus};

unsafe extern "C" fn is_alive(_: *const c_void, _: *mut u8) -> AmmlStatus {
    AmmlStatus::OK
}
unsafe extern "C" fn health(_: *const c_void, _: *mut i32) -> AmmlStatus {
    AmmlStatus::OK
}

fn table() -> AmmlActorTable {
    AmmlActorTable {
        size: AmmlActorTable::SIZE_V3,
        version: 3,
        is_alive,
        position: None,
        eye_position: None,
        rotation: None,
        dimension: None,
        dimension_id: None,
        head_position: None,
        block_source: None,
        health: Some(health),
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
