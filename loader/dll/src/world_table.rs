use std::ffi::c_void;

use amml_abi::{
    AmmlAabb, AmmlActorBox, AmmlActorEntry, AmmlApplyStats, AmmlBlockHandle, AmmlBlockHit, AmmlBlockInfo,
    AmmlBlockMatch, AmmlBlockPos, AmmlChunkLoad, AmmlColumnReadStats, AmmlColumnRequest, AmmlColumnSample,
    AmmlItemSnapshot, AmmlKindsResult, AmmlRayOptions, AmmlShapeOp, AmmlStateEntry, AmmlStateValue, AmmlStatus,
    AmmlStr, AmmlSurfaceSample, AmmlVec3, AmmlWorldTable,
};
use amml_runtime::BlockTagValue;

use crate::{
    BlockListener, BlockReadAdapter, BlockStateAdapter, Boundary, CameraAdapter, KindsAdapter, LoaderState,
    ShapesAdapter, WorldAdapter,
};

const TABLE_VERSION: u32 = 11;

pub(crate) static WORLD: AmmlWorldTable = AmmlWorldTable {
    size: AmmlWorldTable::SIZE_V11,
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
    block_from_state: Some(WorldTable::block_from_state),
    block_rotated: Some(WorldTable::block_rotated),
    block_handle_name: Some(WorldTable::block_handle_name),
    block_handle_item: Some(WorldTable::block_handle_item),
    block_at: Some(WorldTable::block_at),
    block_info: Some(WorldTable::block_info),
    block_built_over: Some(WorldTable::block_built_over),
    block_item: Some(WorldTable::block_item),
    block_handle_of: Some(WorldTable::block_handle_of),
    block_changed_state: Some(WorldTable::block_changed_state),
    kinds_in_box: Some(WorldTable::kinds_in_box),
    actors_in_v2: Some(WorldTable::actors_in_v2),
    biome_at: Some(WorldTable::biome_at),
};

/// The world table behind `AmmlServices::world`, and its `extern "C"` functions. Each function checks its
/// pointers and delegates to `WorldAdapter` (or `ShapesAdapter` for the shapes) inside the boundary, so a panic never
/// crosses it. With no adapter installed (a host without one) every call of its functions gives `NOT_AVAILABLE`. The
/// shape, ray and actor calls follow the game thread rule of a `CameraSetup` listener first
/// (`CameraAdapter::check_game_call`). The block state calls delegate to `BlockStateAdapter` and follow that rule too; so do
/// the block read calls (`BlockReadAdapter`), `kinds_in_box` (`KindsAdapter`) and `block_changed_state` (`BlockListener`).
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

    unsafe extern "C" fn actors_in_v2(
        source: *const c_void,
        aabb: AmmlAabb,
        out: *mut AmmlActorEntry,
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
            let entries: &mut [AmmlActorEntry] = if cap == 0 {
                &mut []
            } else {
                // SAFETY: `out` is non-null and the caller promises `cap` entries are writable.
                unsafe { std::slice::from_raw_parts_mut(out, cap as usize) }
            };
            let found = CameraAdapter::check_game_call()
                .and_then(|()| WorldAdapter::get().ok_or(AmmlStatus::NOT_AVAILABLE))
                .and_then(|adapter| adapter.actors_in_entries(source, &aabb, entries));
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

    unsafe extern "C" fn biome_at(source: *const c_void, pos: AmmlBlockPos, out: *mut u16) -> AmmlStatus {
        Boundary::guard(|| {
            if out.is_null() {
                return AmmlStatus::INVALID_ARGUMENT;
            }
            let found = CameraAdapter::check_game_call()
                .and_then(|()| WorldAdapter::get().ok_or(AmmlStatus::NOT_AVAILABLE))
                .and_then(|adapter| adapter.biome_at(source, pos));
            match found {
                Ok(id) => {
                    // SAFETY: `out` is non-null and the caller promises it is writable.
                    unsafe { out.write(id) };
                    AmmlStatus::OK
                }
                Err(status) => status,
            }
        })
    }

    unsafe extern "C" fn block_from_state(
        module: AmmlStr,
        name: AmmlStr,
        states: *const AmmlStateEntry,
        count: u32,
        out: *mut AmmlBlockMatch,
    ) -> AmmlStatus {
        Boundary::guard(|| {
            if out.is_null() || (states.is_null() && count != 0) {
                return AmmlStatus::INVALID_ARGUMENT;
            }
            let owner = match Self::owner(module) {
                Ok(owner) => owner,
                Err(status) => return status,
            };
            // SAFETY: the caller promises `name` is valid for the call.
            let name = match unsafe { name.as_str() } {
                Ok(name) => name,
                Err(status) => return status,
            };
            let entries: &[AmmlStateEntry] = if count == 0 {
                &[]
            } else {
                // SAFETY: `states` is non-null and the caller promises `count` entries are readable for the call.
                unsafe { std::slice::from_raw_parts(states, count as usize) }
            };
            let states = match Self::states(entries) {
                Ok(states) => states,
                Err(status) => return status,
            };
            let found = CameraAdapter::check_game_call()
                .and_then(|()| BlockStateAdapter::get().ok_or(AmmlStatus::NOT_AVAILABLE))
                .and_then(|adapter| adapter.from_state(owner, name, &states));
            let found = match found {
                Ok(Some((block, exact))) => AmmlBlockMatch { block, exact: u8::from(exact), _reserved: [0; 7] },
                Ok(None) => AmmlBlockMatch { block: AmmlBlockHandle::NONE, exact: 0, _reserved: [0; 7] },
                Err(status) => return status,
            };
            // SAFETY: `out` is non-null and the caller promises it is writable.
            unsafe { out.write(found) };
            AmmlStatus::OK
        })
    }

    unsafe extern "C" fn block_rotated(
        module: AmmlStr,
        block: AmmlBlockHandle,
        quarters: u8,
        out: *mut AmmlBlockHandle,
    ) -> AmmlStatus {
        Boundary::guard(|| {
            if out.is_null() {
                return AmmlStatus::INVALID_ARGUMENT;
            }
            let owner = match Self::owner(module) {
                Ok(owner) => owner,
                Err(status) => return status,
            };
            let turned = CameraAdapter::check_game_call()
                .and_then(|()| BlockStateAdapter::get().ok_or(AmmlStatus::NOT_AVAILABLE))
                .and_then(|adapter| adapter.rotated(owner, block, quarters));
            match turned {
                Ok(handle) => {
                    // SAFETY: `out` is non-null and the caller promises it is writable.
                    unsafe { out.write(handle) };
                    AmmlStatus::OK
                }
                Err(status) => status,
            }
        })
    }

    unsafe extern "C" fn block_handle_name(
        block: AmmlBlockHandle,
        buf: *mut u8,
        cap: usize,
        out_len: *mut usize,
    ) -> AmmlStatus {
        Boundary::guard(|| {
            if out_len.is_null() || (buf.is_null() && cap != 0) {
                return AmmlStatus::INVALID_ARGUMENT;
            }
            let name = CameraAdapter::check_game_call()
                .and_then(|()| BlockStateAdapter::get().ok_or(AmmlStatus::NOT_AVAILABLE))
                .and_then(|adapter| adapter.name(block));
            match name {
                Ok(name) => {
                    // SAFETY: the caller promises `buf` is writable for `cap` bytes and `out_len` is writable.
                    unsafe { Self::copy_text(name.as_bytes(), buf, cap, out_len) };
                    AmmlStatus::OK
                }
                Err(status) => status,
            }
        })
    }

    unsafe extern "C" fn block_handle_item(
        block: AmmlBlockHandle,
        out: *mut AmmlItemSnapshot,
        name_buf: *mut u8,
        name_cap: usize,
        name_len: *mut usize,
    ) -> AmmlStatus {
        Boundary::guard(|| {
            if out.is_null() || name_len.is_null() || (name_buf.is_null() && name_cap != 0) {
                return AmmlStatus::INVALID_ARGUMENT;
            }
            // SAFETY: `out` is non-null and the caller promises its leading `size` field is readable.
            let size = unsafe { out.cast::<u32>().read_unaligned() };
            if size < AmmlItemSnapshot::SIZE_V1 {
                return AmmlStatus::INVALID_ARGUMENT;
            }
            let item = CameraAdapter::check_game_call()
                .and_then(|()| BlockStateAdapter::get().ok_or(AmmlStatus::NOT_AVAILABLE))
                .and_then(|adapter| adapter.item(block));
            match item {
                Ok((snapshot, name)) => {
                    let name = name.unwrap_or_default();
                    // SAFETY: `out` holds at least a v1 snapshot (checked above), which the loader writes in full; the
                    // caller promises the name buffer and `name_len` are writable.
                    unsafe {
                        out.write(snapshot);
                        Self::copy_text(name.as_bytes(), name_buf, name_cap, name_len);
                    }
                    AmmlStatus::OK
                }
                Err(status) => status,
            }
        })
    }

    unsafe extern "C" fn block_at(
        source: *const c_void,
        pos: *const AmmlBlockPos,
        out: *mut *const c_void,
    ) -> AmmlStatus {
        Boundary::guard(|| {
            if pos.is_null() || out.is_null() {
                return AmmlStatus::INVALID_ARGUMENT;
            }
            // SAFETY: `pos` is non-null and the caller promises it is readable.
            let pos = unsafe { pos.read() };
            let found = CameraAdapter::check_game_call()
                .and_then(|()| BlockReadAdapter::get().ok_or(AmmlStatus::NOT_AVAILABLE))
                .and_then(|adapter| adapter.block_at(source, [pos.x, pos.y, pos.z]));
            match found {
                Ok(handle) => {
                    // SAFETY: `out` is non-null and the caller promises it is writable.
                    unsafe { out.write(std::ptr::with_exposed_provenance(handle)) };
                    AmmlStatus::OK
                }
                Err(status) => status,
            }
        })
    }

    unsafe extern "C" fn block_info(block: *const c_void, out: *mut AmmlBlockInfo) -> AmmlStatus {
        Boundary::guard(|| {
            if out.is_null() {
                return AmmlStatus::INVALID_ARGUMENT;
            }
            let info = CameraAdapter::check_game_call()
                .and_then(|()| BlockReadAdapter::get().ok_or(AmmlStatus::NOT_AVAILABLE))
                .and_then(|adapter| adapter.info(block));
            match info {
                Ok(info) => {
                    // SAFETY: `out` is non-null and the caller promises it is writable.
                    unsafe { out.write(info) };
                    AmmlStatus::OK
                }
                Err(status) => status,
            }
        })
    }

    unsafe extern "C" fn block_built_over(
        source: *const c_void,
        block: *const c_void,
        pos: *const AmmlBlockPos,
        out: *mut u8,
    ) -> AmmlStatus {
        Boundary::guard(|| {
            if pos.is_null() || out.is_null() {
                return AmmlStatus::INVALID_ARGUMENT;
            }
            // SAFETY: `pos` is non-null and the caller promises it is readable.
            let pos = unsafe { pos.read() };
            let found = CameraAdapter::check_game_call()
                .and_then(|()| BlockReadAdapter::get().ok_or(AmmlStatus::NOT_AVAILABLE))
                .and_then(|adapter| adapter.built_over(source, block, [pos.x, pos.y, pos.z]));
            match found {
                Ok(over) => {
                    // SAFETY: `out` is non-null and the caller promises it is writable.
                    unsafe { out.write(u8::from(over)) };
                    AmmlStatus::OK
                }
                Err(status) => status,
            }
        })
    }

    unsafe extern "C" fn block_item(
        block: *const c_void,
        out: *mut AmmlItemSnapshot,
        name_buf: *mut u8,
        name_cap: usize,
        name_len: *mut usize,
    ) -> AmmlStatus {
        Boundary::guard(|| {
            if out.is_null() || name_len.is_null() || (name_buf.is_null() && name_cap != 0) {
                return AmmlStatus::INVALID_ARGUMENT;
            }
            // SAFETY: `out` is non-null and the caller promises its leading `size` field is readable.
            let size = unsafe { out.cast::<u32>().read_unaligned() };
            if size < AmmlItemSnapshot::SIZE_V1 {
                return AmmlStatus::INVALID_ARGUMENT;
            }
            let item = CameraAdapter::check_game_call()
                .and_then(|()| BlockReadAdapter::get().ok_or(AmmlStatus::NOT_AVAILABLE))
                .and_then(|adapter| adapter.item(block));
            match item {
                Ok((snapshot, name)) => {
                    let name = name.unwrap_or_default();
                    // SAFETY: `out` holds at least a v1 snapshot (checked above), which the loader writes in full; the
                    // caller promises the name buffer and `name_len` are writable.
                    unsafe {
                        out.write(snapshot);
                        Self::copy_text(name.as_bytes(), name_buf, name_cap, name_len);
                    }
                    AmmlStatus::OK
                }
                Err(status) => status,
            }
        })
    }

    unsafe extern "C" fn block_handle_of(
        module: AmmlStr,
        block: *const c_void,
        out: *mut AmmlBlockHandle,
    ) -> AmmlStatus {
        Boundary::guard(|| {
            if out.is_null() {
                return AmmlStatus::INVALID_ARGUMENT;
            }
            let owner = match Self::owner(module) {
                Ok(owner) => owner,
                Err(status) => return status,
            };
            let handle = CameraAdapter::check_game_call()
                .and_then(|()| BlockReadAdapter::get().ok_or(AmmlStatus::NOT_AVAILABLE))
                .and_then(|adapter| adapter.handle_of(owner, block));
            match handle {
                Ok(handle) => {
                    // SAFETY: `out` is non-null and the caller promises it is writable.
                    unsafe { out.write(handle) };
                    AmmlStatus::OK
                }
                Err(status) => status,
            }
        })
    }

    unsafe extern "C" fn block_changed_state(module: AmmlStr, token: u64, out: *mut AmmlBlockHandle) -> AmmlStatus {
        Boundary::guard(|| {
            if out.is_null() {
                return AmmlStatus::INVALID_ARGUMENT;
            }
            let owner = match Self::owner(module) {
                Ok(owner) => owner,
                Err(status) => return status,
            };
            let handle =
                BlockListener::with(Err(AmmlStatus::NOT_AVAILABLE), |listener| listener.new_state(owner, token));
            match handle {
                Ok(handle) => {
                    // SAFETY: `out` is non-null and the caller promises it is writable.
                    unsafe { out.write(handle) };
                    AmmlStatus::OK
                }
                Err(status) => status,
            }
        })
    }

    unsafe extern "C" fn kinds_in_box(
        source: *const c_void,
        min: *const AmmlBlockPos,
        max: *const AmmlBlockPos,
        ids: *mut u32,
        cap: u32,
        out: *mut AmmlKindsResult,
    ) -> AmmlStatus {
        Boundary::guard(|| {
            if min.is_null() || max.is_null() || out.is_null() || (ids.is_null() && cap != 0) {
                return AmmlStatus::INVALID_ARGUMENT;
            }
            // SAFETY: `min` and `max` are non-null and the caller promises they are readable.
            let (min, max) = unsafe { (min.read(), max.read()) };
            let room: &mut [u32] = if cap == 0 {
                &mut []
            } else {
                // SAFETY: `ids` is non-null and the caller promises `cap` ids are writable.
                unsafe { std::slice::from_raw_parts_mut(ids, cap as usize) }
            };
            let found = CameraAdapter::check_game_call()
                .and_then(|()| KindsAdapter::get().ok_or(AmmlStatus::NOT_AVAILABLE))
                .and_then(|adapter| adapter.kinds_in_box(source, min, max, room));
            match found {
                Ok(result) => {
                    // SAFETY: `out` is non-null and the caller promises it is writable.
                    unsafe { out.write(result) };
                    AmmlStatus::OK
                }
                Err(status) => status,
            }
        })
    }

    // The block states of `entries` as tag values; `INVALID_ARGUMENT` for a text that is not UTF-8 or an unknown kind.
    fn states(entries: &[AmmlStateEntry]) -> Result<Vec<(&str, BlockTagValue<'_>)>, AmmlStatus> {
        entries
            .iter()
            .map(|entry| {
                // SAFETY: the caller promises the texts of every entry are valid for the call.
                let name = unsafe { entry.name.as_str() }?;
                let value = match entry.value.kind {
                    AmmlStateValue::INT => BlockTagValue::Int(entry.value.int),
                    AmmlStateValue::BOOL => BlockTagValue::Bool(entry.value.int != 0),
                    // SAFETY: as above.
                    AmmlStateValue::TEXT => BlockTagValue::Text(unsafe { entry.value.text.as_str() }?),
                    _ => return Err(AmmlStatus::INVALID_ARGUMENT),
                };
                Ok((name, value))
            })
            .collect()
    }

    // Copies `bytes` to `buf` (at most `cap` of them) and writes their full length to `out_len`.
    //
    // Safety: `buf` is writable for `cap` bytes (or `cap` is 0) and `out_len` is non-null and writable.
    unsafe fn copy_text(bytes: &[u8], buf: *mut u8, cap: usize, out_len: *mut usize) {
        let copied = bytes.len().min(cap);
        if copied > 0 {
            // SAFETY: the caller promises `buf` is writable for `cap` bytes, and `copied <= cap`.
            unsafe { std::ptr::copy_nonoverlapping(bytes.as_ptr(), buf, copied) };
        }
        // SAFETY: the caller promises `out_len` is writable.
        unsafe { out_len.write(bytes.len()) };
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
