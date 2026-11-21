use std::ffi::c_void;

use amml_abi::{
    AmmlAabb, AmmlActorBox, AmmlApplyStats, AmmlBlockHit, AmmlChunkLoad, AmmlColumnReadStats, AmmlColumnRequest,
    AmmlColumnSample, AmmlRayOptions, AmmlShapeOp, AmmlStatus, AmmlStr, AmmlSurfaceSample, AmmlVec3, AmmlWorldTable,
};

use crate::{Boundary, CameraAdapter, LoaderState, ShapesAdapter, WorldAdapter};

const TABLE_VERSION: u32 = 6;

pub(crate) static WORLD: AmmlWorldTable = AmmlWorldTable {
    size: AmmlWorldTable::SIZE_V6,
    version: TABLE_VERSION,
    sky_darken: WorldTable::sky_darken,
    chunk: Some(WorldTable::chunk),
    chunk_load_state: Some(WorldTable::chunk_load_state),
    read_columns: Some(WorldTable::read_columns),
    block_type_name: Some(WorldTable::block_type_name),
    biome_name: Some(WorldTable::biome_name),
    scan_surfaces: Some(WorldTable::scan_surfaces),
    shapes_apply: Some(WorldTable::shapes_apply),
    shapes_max_batch: Some(WorldTable::shapes_max_batch),
    shapes_target_epoch: Some(WorldTable::shapes_target_epoch),
    raycast: Some(WorldTable::raycast),
    actors_in: Some(WorldTable::actors_in),
};

/// The world table behind `AmmlServices::world`, and its `extern "C"` functions. Each function checks its
/// pointers and delegates to `WorldAdapter` (or `ShapesAdapter` for the shapes) inside the boundary, so a panic never
/// crosses it. With no adapter installed (a host without one) every call of its functions gives `NOT_AVAILABLE`. The
/// shape, ray and actor calls follow the game thread rule of a `CameraSetup` listener first
/// (`CameraAdapter::check_game_call`).
pub struct WorldTable;

impl WorldTable {
    unsafe extern "C" fn sky_darken(dimension: *const c_void, out: *mut u8) -> AmmlStatus {
        Boundary::guard(|| {
            if out.is_null() {
                return AmmlStatus::INVALID_ARGUMENT;
            }
            let adapter = WorldAdapter::get().ok_or(AmmlStatus::NOT_AVAILABLE);
            match adapter.and_then(|adapter| adapter.sky_darken(dimension)) {
                Ok(value) => {
                    // SAFETY: `out` is non-null and the caller promises it is writable.
                    unsafe { out.write(value) };
                    AmmlStatus::OK
                }
                Err(status) => status,
            }
        })
    }

    unsafe extern "C" fn chunk(source: *const c_void, x: i32, z: i32, out: *mut *const c_void) -> AmmlStatus {
        Boundary::guard(|| {
            if out.is_null() {
                return AmmlStatus::INVALID_ARGUMENT;
            }
            let adapter = WorldAdapter::get().ok_or(AmmlStatus::NOT_AVAILABLE);
            match adapter.and_then(|adapter| adapter.chunk(source, x, z)) {
                Ok(handle) => {
                    // SAFETY: `out` is non-null and the caller promises it is writable.
                    unsafe { out.write(std::ptr::with_exposed_provenance(handle)) };
                    AmmlStatus::OK
                }
                Err(status) => status,
            }
        })
    }

    unsafe extern "C" fn chunk_load_state(chunk: *const c_void, out: *mut AmmlChunkLoad) -> AmmlStatus {
        Boundary::guard(|| {
            if out.is_null() {
                return AmmlStatus::INVALID_ARGUMENT;
            }
            let adapter = WorldAdapter::get().ok_or(AmmlStatus::NOT_AVAILABLE);
            match adapter.and_then(|adapter| adapter.chunk_load_state(chunk)) {
                Ok(value) => {
                    // SAFETY: `out` is non-null and the caller promises it is writable.
                    unsafe { out.write(value) };
                    AmmlStatus::OK
                }
                Err(status) => status,
            }
        })
    }

    unsafe extern "C" fn read_columns(
        chunk: *const c_void,
        request: *const AmmlColumnRequest,
        out: *mut AmmlColumnSample,
        len: u32,
        stats: *mut AmmlColumnReadStats,
    ) -> AmmlStatus {
        Boundary::guard(|| {
            if request.is_null() || out.is_null() || stats.is_null() || len != WorldAdapter::COLUMNS {
                return AmmlStatus::INVALID_ARGUMENT;
            }
            // SAFETY: `request` is non-null and the caller promises it is readable.
            let request = unsafe { request.read() };
            // SAFETY: `out` is non-null and the caller promises `len` samples (256, checked above) are writable.
            let samples = unsafe { std::slice::from_raw_parts_mut(out, len as usize) };
            let adapter = WorldAdapter::get().ok_or(AmmlStatus::NOT_AVAILABLE);
            match adapter.and_then(|adapter| adapter.read_columns(chunk, &request, samples)) {
                Ok(value) => {
                    // SAFETY: `stats` is non-null and the caller promises it is writable.
                    unsafe { stats.write(value) };
                    AmmlStatus::OK
                }
                Err(status) => status,
            }
        })
    }

    unsafe extern "C" fn block_type_name(id: u32, buf: *mut u8, cap: usize, out_len: *mut usize) -> AmmlStatus {
        Boundary::guard(|| {
            if out_len.is_null() || (buf.is_null() && cap != 0) {
                return AmmlStatus::INVALID_ARGUMENT;
            }
            let adapter = WorldAdapter::get().ok_or(AmmlStatus::NOT_AVAILABLE);
            match adapter.and_then(|adapter| adapter.block_type_name(id)) {
                Ok(name) => {
                    let bytes = name.as_bytes();
                    let copied = bytes.len().min(cap);
                    if copied > 0 {
                        // SAFETY: the caller promises `buf` is writable for `cap` bytes, and `copied <= cap`.
                        unsafe { std::ptr::copy_nonoverlapping(bytes.as_ptr(), buf, copied) };
                    }
                    // SAFETY: `out_len` is non-null and the caller promises it is writable.
                    unsafe { out_len.write(bytes.len()) };
                    AmmlStatus::OK
                }
                Err(status) => status,
            }
        })
    }

    unsafe extern "C" fn biome_name(id: u16, buf: *mut u8, cap: usize, out_len: *mut usize) -> AmmlStatus {
        Boundary::guard(|| {
            if out_len.is_null() || (buf.is_null() && cap != 0) {
                return AmmlStatus::INVALID_ARGUMENT;
            }
            let adapter = WorldAdapter::get().ok_or(AmmlStatus::NOT_AVAILABLE);
            match adapter.and_then(|adapter| adapter.biome_name(id)) {
                Ok(name) => {
                    let bytes = name.as_bytes();
                    let copied = bytes.len().min(cap);
                    if copied > 0 {
                        // SAFETY: the caller promises `buf` is writable for `cap` bytes, and `copied <= cap`.
                        unsafe { std::ptr::copy_nonoverlapping(bytes.as_ptr(), buf, copied) };
                    }
                    // SAFETY: `out_len` is non-null and the caller promises it is writable.
                    unsafe { out_len.write(bytes.len()) };
                    AmmlStatus::OK
                }
                Err(status) => status,
            }
        })
    }

    unsafe extern "C" fn scan_surfaces(
        source: *const c_void,
        x: i32,
        z: i32,
        y_min: i32,
        y_max: i32,
        out: *mut AmmlSurfaceSample,
        len: u32,
        out_count: *mut u32,
    ) -> AmmlStatus {
        Boundary::guard(|| {
            if out_count.is_null() || (out.is_null() && len != 0) {
                return AmmlStatus::INVALID_ARGUMENT;
            }
            let samples: &mut [AmmlSurfaceSample] = if len == 0 {
                &mut []
            } else {
                // SAFETY: `out` is non-null and the caller promises `len` samples are writable.
                unsafe { std::slice::from_raw_parts_mut(out, len as usize) }
            };
            let adapter = WorldAdapter::get().ok_or(AmmlStatus::NOT_AVAILABLE);
            match adapter.and_then(|adapter| adapter.scan_surfaces(source, (x, z), (y_min, y_max), samples)) {
                Ok(count) => {
                    // SAFETY: `out_count` is non-null and the caller promises it is writable.
                    unsafe { out_count.write(count) };
                    AmmlStatus::OK
                }
                Err(status) => status,
            }
        })
    }

    unsafe extern "C" fn shapes_apply(
        module: AmmlStr,
        dimension: i32,
        ops: *const AmmlShapeOp,
        count: u32,
        out: *mut AmmlApplyStats,
    ) -> AmmlStatus {
        Boundary::guard(|| {
            if ops.is_null() && count != 0 {
                return AmmlStatus::INVALID_ARGUMENT;
            }
            let ops: &[AmmlShapeOp] = if count == 0 {
                &[]
            } else {
                // SAFETY: `ops` is non-null and the caller promises `count` ops are readable for the call.
                unsafe { std::slice::from_raw_parts(ops, count as usize) }
            };
            let owner = match Self::owner(module) {
                Ok(owner) => owner,
                Err(status) => return status,
            };
            let applied = CameraAdapter::check_game_call()
                .and_then(|()| ShapesAdapter::get().ok_or(AmmlStatus::NOT_AVAILABLE))
                .and_then(|adapter| adapter.apply(owner, dimension, ops));
            match applied {
                Ok(stats) => {
                    if !out.is_null() {
                        // SAFETY: `out` is non-null and the caller promises it is writable.
                        unsafe { out.write(stats) };
                    }
                    AmmlStatus::OK
                }
                Err(status) => status,
            }
        })
    }

    unsafe extern "C" fn shapes_max_batch() -> u32 {
        ShapesAdapter::max_batch()
    }

    unsafe extern "C" fn shapes_target_epoch(out: *mut u32) -> AmmlStatus {
        Boundary::guard(|| {
            if out.is_null() {
                return AmmlStatus::INVALID_ARGUMENT;
            }
            match ShapesAdapter::get().ok_or(AmmlStatus::NOT_AVAILABLE).and_then(ShapesAdapter::target_epoch) {
                Ok(epoch) => {
                    // SAFETY: `out` is non-null and the caller promises it is writable.
                    unsafe { out.write(epoch) };
                    AmmlStatus::OK
                }
                Err(status) => status,
            }
        })
    }

    unsafe extern "C" fn raycast(
        source: *const c_void,
        from: AmmlVec3,
        to: AmmlVec3,
        options: AmmlRayOptions,
        out: *mut AmmlBlockHit,
    ) -> AmmlStatus {
        Boundary::guard(|| {
            if out.is_null() || !Self::finite(&from) || !Self::finite(&to) {
                return AmmlStatus::INVALID_ARGUMENT;
            }
            let found = CameraAdapter::check_game_call()
                .and_then(|()| WorldAdapter::get().ok_or(AmmlStatus::NOT_AVAILABLE))
                .and_then(|adapter| adapter.raycast(source, from, to, options));
            match found {
                Ok(hit) => {
                    // SAFETY: `out` is non-null and the caller promises it is writable.
                    unsafe { out.write(hit) };
                    AmmlStatus::OK
                }
                Err(status) => status,
            }
        })
    }

    unsafe extern "C" fn actors_in(
        source: *const c_void,
        aabb: AmmlAabb,
        out: *mut AmmlActorBox,
        cap: u32,
        out_count: *mut u32,
    ) -> AmmlStatus {
        Boundary::guard(|| {
            if out_count.is_null()
                || (out.is_null() && cap != 0)
                || !Self::finite(&aabb.min)
                || !Self::finite(&aabb.max)
            {
                return AmmlStatus::INVALID_ARGUMENT;
            }
            let boxes: &mut [AmmlActorBox] = if cap == 0 {
                &mut []
            } else {
                // SAFETY: `out` is non-null and the caller promises `cap` boxes are writable.
                unsafe { std::slice::from_raw_parts_mut(out, cap as usize) }
            };
            let found = CameraAdapter::check_game_call()
                .and_then(|()| WorldAdapter::get().ok_or(AmmlStatus::NOT_AVAILABLE))
                .and_then(|adapter| adapter.actors_in(source, &aabb, boxes));
            match found {
                Ok(count) => {
                    // SAFETY: `out_count` is non-null and the caller promises it is writable.
                    unsafe { out_count.write(count) };
                    AmmlStatus::OK
                }
                Err(status) => status,
            }
        })
    }

    fn finite(point: &AmmlVec3) -> bool {
        point.x.is_finite() && point.y.is_finite() && point.z.is_finite()
    }

    // The index of the live mod `module`.
    fn owner(module: AmmlStr) -> Result<usize, AmmlStatus> {
        let state = LoaderState::get().ok_or(AmmlStatus::FAILED)?;
        // SAFETY: the caller promises `module` is valid for the call.
        let module = unsafe { module.as_str() }?;
        state.live_mod(module)
    }
}
