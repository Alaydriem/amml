use core::ffi::c_void;

use amml_abi::{
    AmmlAabb, AmmlActorBox, AmmlApplyStats, AmmlChunkLoad, AmmlShapeOp, AmmlStatus, AmmlStr, AmmlSurfaceSample,
    AmmlWorldTable,
};

unsafe extern "C" fn sky_darken(_: *const c_void, _: *mut u8) -> AmmlStatus {
    AmmlStatus::OK
}

unsafe extern "C" fn chunk(_: *const c_void, _: i32, _: i32, _: *mut *const c_void) -> AmmlStatus {
    AmmlStatus::OK
}

unsafe extern "C" fn load_state(_: *const c_void, _: *mut AmmlChunkLoad) -> AmmlStatus {
    AmmlStatus::OK
}

unsafe extern "C" fn scan(
    _: *const c_void,
    _: i32,
    _: i32,
    _: i32,
    _: i32,
    _: *mut AmmlSurfaceSample,
    _: u32,
    _: *mut u32,
) -> AmmlStatus {
    AmmlStatus::OK
}

unsafe extern "C" fn shapes_apply(
    _: AmmlStr,
    _: i32,
    _: *const AmmlShapeOp,
    _: u32,
    _: *mut AmmlApplyStats,
) -> AmmlStatus {
    AmmlStatus::OK
}

unsafe extern "C" fn shapes_max_batch() -> u32 {
    800
}

unsafe extern "C" fn actors_in(_: *const c_void, _: AmmlAabb, _: *mut AmmlActorBox, _: u32, _: *mut u32) -> AmmlStatus {
    AmmlStatus::OK
}

fn table() -> AmmlWorldTable {
    AmmlWorldTable {
        size: AmmlWorldTable::SIZE_V6,
        version: 6,
        sky_darken,
        chunk: Some(chunk),
        chunk_load_state: Some(load_state),
        read_columns: None,
        block_type_name: None,
        biome_name: None,
        scan_surfaces: Some(scan),
        shapes_apply: Some(shapes_apply),
        shapes_max_batch: Some(shapes_max_batch),
        shapes_target_epoch: None,
        raycast: None,
        actors_in: Some(actors_in),
    }
}

#[test]
fn world_table_read_respects_size() {
    let mut table = table();
    // SAFETY: the table lives for the call.
    assert!(unsafe { AmmlWorldTable::read(&raw const table) }.is_some());
    table.size = AmmlWorldTable::SIZE_V1 - 8;
    // SAFETY: the table lives for the call.
    assert!(unsafe { AmmlWorldTable::read(&raw const table) }.is_none());
    // SAFETY: a null table is never read.
    assert!(unsafe { AmmlWorldTable::read(core::ptr::null()) }.is_none());
}

#[test]
fn a_v1_world_table_reads_none_for_the_v2_functions() {
    let mut table = table();
    table.size = AmmlWorldTable::SIZE_V1;

    // SAFETY: the table lives for the call and the v2 bytes past `size` are never trusted.
    let read = unsafe { AmmlWorldTable::read(&raw const table) }.unwrap();

    assert!(read.chunk.is_none() && read.chunk_load_state.is_none());
    assert!(read.read_columns.is_none() && read.block_type_name.is_none());
}

#[test]
fn a_v2_world_table_reads_its_functions() {
    let table = table();

    // SAFETY: the table lives for the call.
    let read = unsafe { AmmlWorldTable::read(&raw const table) }.unwrap();

    assert!(read.chunk.is_some() && read.chunk_load_state.is_some());
}

#[test]
fn a_v2_world_table_reads_none_for_the_biome_name() {
    let mut table = table();
    table.size = AmmlWorldTable::SIZE_V2;

    // SAFETY: the table lives for the call and the v3 bytes past `size` are never trusted.
    let read = unsafe { AmmlWorldTable::read(&raw const table) }.unwrap();

    assert!(read.biome_name.is_none());
}

#[test]
fn a_v3_world_table_reads_none_for_scan_surfaces() {
    let mut table = table();
    table.size = AmmlWorldTable::SIZE_V3;

    // SAFETY: the table lives for the call and the v4 bytes past `size` are never trusted.
    let read = unsafe { AmmlWorldTable::read(&raw const table) }.unwrap();

    assert!(read.scan_surfaces.is_none());
}

#[test]
fn a_v4_world_table_reads_none_for_the_shape_functions() {
    let mut table = table();
    table.size = AmmlWorldTable::SIZE_V4;

    // SAFETY: the table lives for the call and the v5 bytes past `size` are never trusted.
    let read = unsafe { AmmlWorldTable::read(&raw const table) }.unwrap();

    assert!(read.scan_surfaces.is_some());
    assert!(read.shapes_apply.is_none() && read.shapes_max_batch.is_none() && read.shapes_target_epoch.is_none());
}

#[test]
fn a_v5_world_table_reads_the_shape_functions() {
    let table = table();

    // SAFETY: the table lives for the call.
    let read = unsafe { AmmlWorldTable::read(&raw const table) }.unwrap();

    assert!(read.shapes_apply.is_some() && read.shapes_max_batch.is_some());
}

#[test]
fn a_v5_world_table_reads_none_for_the_ray_functions_and_a_v6_reads_them() {
    let mut table = table();
    table.size = AmmlWorldTable::SIZE_V5;

    // SAFETY: the table lives for the call and the v6 bytes past `size` are never trusted.
    let read = unsafe { AmmlWorldTable::read(&raw const table) }.unwrap();

    assert!(read.shapes_target_epoch.is_none() && read.raycast.is_none() && read.actors_in.is_none());
    let full = self::table();
    // SAFETY: the table lives for the call.
    assert!(unsafe { AmmlWorldTable::read(&raw const full) }.unwrap().actors_in.is_some());
}
