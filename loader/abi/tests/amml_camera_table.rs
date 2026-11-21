use std::ffi::c_void;

use amml_abi::{AmmlCameraKind, AmmlCameraTable, AmmlPose, AmmlProjection, AmmlStatus, AmmlStr, AmmlVec3};

unsafe extern "C" fn set_pose(_: AmmlStr, _: u64, _: *const AmmlPose) -> AmmlStatus {
    AmmlStatus::OK
}

unsafe extern "C" fn rendered_position(_: u64, _: *mut AmmlVec3) -> AmmlStatus {
    AmmlStatus::OK
}

unsafe extern "C" fn projection(_: u64, _: *mut AmmlProjection) -> AmmlStatus {
    AmmlStatus::OK
}

unsafe extern "C" fn set_free(_: AmmlStr, _: *const AmmlPose) -> AmmlStatus {
    AmmlStatus::OK
}

unsafe extern "C" fn release_free(_: AmmlStr) -> AmmlStatus {
    AmmlStatus::OK
}

unsafe extern "C" fn free_available(_: *mut u8) -> AmmlStatus {
    AmmlStatus::OK
}

unsafe extern "C" fn active_camera(_: *const c_void, _: *mut AmmlCameraKind) -> AmmlStatus {
    AmmlStatus::OK
}

fn table() -> AmmlCameraTable {
    AmmlCameraTable {
        size: AmmlCameraTable::SIZE_V3,
        version: 3,
        set_pose,
        rendered_position,
        projection,
        set_free: Some(set_free),
        release_free: Some(release_free),
        free_available: Some(free_available),
        active_camera: Some(active_camera),
    }
}

#[test]
fn a_v1_camera_table_reads_none_for_the_free_camera_functions() {
    let mut table = table();
    table.size = AmmlCameraTable::SIZE_V1;

    // SAFETY: the table lives for the call and the v2 bytes past `size` are never trusted.
    let read = unsafe { AmmlCameraTable::read(&raw const table) }.unwrap();

    assert!(read.set_free.is_none() && read.release_free.is_none() && read.free_available.is_none());
    assert!(read.active_camera.is_none());
}

#[test]
fn a_v2_camera_table_reads_none_for_the_active_camera_function() {
    let mut table = table();
    table.size = AmmlCameraTable::SIZE_V2;

    // SAFETY: the table lives for the call and the v3 bytes past `size` are never trusted.
    let read = unsafe { AmmlCameraTable::read(&raw const table) }.unwrap();

    assert!(read.free_available.is_some());
    assert!(read.active_camera.is_none());
}

#[test]
fn a_v3_camera_table_reads_the_free_camera_and_active_camera_functions() {
    let table = table();

    // SAFETY: the table lives for the call.
    let read = unsafe { AmmlCameraTable::read(&raw const table) }.unwrap();

    assert_eq!(AmmlCameraTable::SIZE_V2, 56);
    assert_eq!(AmmlCameraTable::SIZE_V3, 64);
    assert!(read.set_free.is_some() && read.release_free.is_some() && read.free_available.is_some());
    assert!(read.active_camera.is_some());
}
