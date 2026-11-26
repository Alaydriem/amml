use amml_abi::{
    AmmlBlockEntityGhost, AmmlBlockGrid, AmmlBlockHandle, AmmlBlockPos, AmmlColorQuad, AmmlGhostEntityOptions,
    AmmlMeshBuilt, AmmlMeshGroup, AmmlMeshId, AmmlMeshStyle, AmmlMesherOptions, AmmlRenderTable, AmmlStatus, AmmlStr,
    AmmlTransform, AmmlVec3,
};

use crate::{BlockEntityAdapter, Boundary, LoaderState, WorldRenderAdapter};

const TABLE_VERSION: u32 = 6;

pub(crate) static RENDER: AmmlRenderTable = AmmlRenderTable {
    size: AmmlRenderTable::SIZE_V6,
    version: TABLE_VERSION,
    canvas_camera_position: RenderTable::canvas_camera_position,
    mesh_free: RenderTable::mesh_free,
    mesh_build: Some(RenderTable::mesh_build),
    mesh_draw: Some(RenderTable::mesh_draw),
    canvas_clip_matrix: Some(RenderTable::canvas_clip_matrix),
    mesher_build: Some(RenderTable::mesher_build),
    mesher_group_begin: Some(RenderTable::mesher_group_begin),
    mesher_group_append: Some(RenderTable::mesher_group_append),
    mesher_group_finish: Some(RenderTable::mesher_group_finish),
    block_entity_create: Some(RenderTable::block_entity_create),
    block_entity_free: Some(RenderTable::block_entity_free),
    block_entity_draw: Some(RenderTable::block_entity_draw),
    mesher_group_discard: Some(RenderTable::mesher_group_discard),
};

/// The render table behind `AmmlServices::render`, and its `extern "C"` functions. Each function checks its pointers
/// and delegates to `WorldRenderAdapter` inside the boundary, so a panic never crosses it. With no world render
/// adapter installed (a host without one) every call gives `NOT_AVAILABLE`. The block entity calls delegate to
/// `BlockEntityAdapter` in the same way.
pub struct RenderTable;

impl RenderTable {
    unsafe extern "C" fn canvas_camera_position(frame: u64, out: *mut AmmlVec3) -> AmmlStatus {
        Boundary::guard(|| {
            if out.is_null() {
                return AmmlStatus::INVALID_ARGUMENT;
            }
            let adapter = WorldRenderAdapter::get().ok_or(AmmlStatus::NOT_AVAILABLE);
            match adapter.and_then(|adapter| adapter.canvas_camera_position(frame)) {
                Ok([x, y, z]) => {
                    // SAFETY: `out` is non-null and the caller promises it is writable.
                    unsafe { out.write(AmmlVec3 { x, y, z }) };
                    AmmlStatus::OK
                }
                Err(status) => status,
            }
        })
    }

    unsafe extern "C" fn canvas_clip_matrix(frame: u64, out: *mut [[f32; 4]; 4]) -> AmmlStatus {
        Boundary::guard(|| {
            if out.is_null() {
                return AmmlStatus::INVALID_ARGUMENT;
            }
            let adapter = WorldRenderAdapter::get().ok_or(AmmlStatus::NOT_AVAILABLE);
            match adapter.and_then(|adapter| adapter.canvas_clip_matrix(frame)) {
                Ok(matrix) => {
                    // SAFETY: `out` is non-null and the caller promises it is writable.
                    unsafe { out.write_unaligned(matrix) };
                    AmmlStatus::OK
                }
                Err(status) => status,
            }
        })
    }

    unsafe extern "C" fn mesh_free(module: AmmlStr, mesh: AmmlMeshId) -> AmmlStatus {
        Boundary::guard(|| {
            let Some(state) = LoaderState::get() else {
                return AmmlStatus::FAILED;
            };
            // SAFETY: the caller promises `module` is valid for the call.
            let module = match unsafe { module.as_str() } {
                Ok(module) => module,
                Err(status) => return status,
            };
            let owner = match state.live_mod(module) {
                Ok(owner) => owner,
                Err(status) => return status,
            };
            match WorldRenderAdapter::get().ok_or(AmmlStatus::NOT_AVAILABLE).and_then(|a| a.mesh_free(owner, mesh)) {
                Ok(()) => AmmlStatus::OK,
                Err(status) => status,
            }
        })
    }

    unsafe extern "C" fn mesh_build(
        module: AmmlStr,
        frame: u64,
        quads: *const AmmlColorQuad,
        count: u32,
        style: AmmlMeshStyle,
        out: *mut AmmlMeshId,
    ) -> AmmlStatus {
        Boundary::guard(|| {
            if out.is_null() || quads.is_null() {
                return AmmlStatus::INVALID_ARGUMENT;
            }
            let owner = match Self::owner(module) {
                Ok(owner) => owner,
                Err(status) => return status,
            };
            let Some(adapter) = WorldRenderAdapter::get() else {
                return AmmlStatus::NOT_AVAILABLE;
            };
            // The count and the style are checked before the quads are read.
            if let Err(status) = adapter.check_build(frame, count as usize, style) {
                return status;
            }
            // SAFETY: the caller promises `count` quads readable at `quads` for the call; `count` is at least 1.
            let quads = unsafe { std::slice::from_raw_parts(quads, count as usize) };
            let result = adapter.mesh_build(owner, frame, quads, style);
            match result {
                Ok(mesh) => {
                    // SAFETY: `out` is non-null and the caller promises it is writable.
                    unsafe { out.write(mesh) };
                    AmmlStatus::OK
                }
                Err(status) => status,
            }
        })
    }

    unsafe extern "C" fn mesh_draw(
        module: AmmlStr,
        frame: u64,
        mesh: AmmlMeshId,
        transform: *const AmmlTransform,
    ) -> AmmlStatus {
        Boundary::guard(|| {
            if transform.is_null() {
                return AmmlStatus::INVALID_ARGUMENT;
            }
            let owner = match Self::owner(module) {
                Ok(owner) => owner,
                Err(status) => return status,
            };
            // SAFETY: `transform` is non-null and the caller promises it is readable for the call.
            let transform = unsafe { transform.read_unaligned() };
            match WorldRenderAdapter::get()
                .ok_or(AmmlStatus::NOT_AVAILABLE)
                .and_then(|adapter| adapter.mesh_draw(owner, frame, mesh, &transform))
            {
                Ok(()) => AmmlStatus::OK,
                Err(status) => status,
            }
        })
    }

    unsafe extern "C" fn mesher_build(
        module: AmmlStr,
        frame: u64,
        grid: *const AmmlBlockGrid,
        options: *const AmmlMesherOptions,
        out: *mut AmmlMeshBuilt,
        skipped: *mut u32,
        skipped_cap: u32,
    ) -> AmmlStatus {
        Boundary::guard(|| {
            if grid.is_null() || options.is_null() || out.is_null() || (skipped.is_null() && skipped_cap > 0) {
                return AmmlStatus::INVALID_ARGUMENT;
            }
            let owner = match Self::owner(module) {
                Ok(owner) => owner,
                Err(status) => return status,
            };
            let Some(adapter) = WorldRenderAdapter::get() else {
                return AmmlStatus::NOT_AVAILABLE;
            };
            // SAFETY: `grid` and `options` are non-null and the caller promises they are readable for the call.
            let (grid, options) = unsafe { (grid.read_unaligned(), options.read_unaligned()) };
            // The frame, the thread and the dims are checked before a cell is read.
            if let Err(status) = adapter.check_mesher(frame, grid.dims, grid.cell_count as usize) {
                return status;
            }
            let (cells, palette) = match Self::grid_slices(&grid) {
                Ok(slices) => slices,
                Err(status) => return status,
            };
            let origin = [grid.origin.x, grid.origin.y, grid.origin.z];
            match adapter.mesher_build(owner, frame, grid.dims, origin, cells, palette, &options) {
                Ok((built, indices)) => {
                    let written = indices.len().min(skipped_cap as usize);
                    // SAFETY: `out` is non-null and writable; `skipped` holds `skipped_cap` writable values when the
                    // cap is not 0, and `written` is at most the cap.
                    unsafe {
                        out.write(built);
                        if written > 0 {
                            std::ptr::copy_nonoverlapping(indices.as_ptr(), skipped, written);
                        }
                    }
                    AmmlStatus::OK
                }
                Err(status) => status,
            }
        })
    }

    unsafe extern "C" fn mesher_group_begin(
        module: AmmlStr,
        frame: u64,
        slot_size: *const [u32; 3],
        out: *mut AmmlMeshGroup,
    ) -> AmmlStatus {
        Boundary::guard(|| {
            if slot_size.is_null() || out.is_null() {
                return AmmlStatus::INVALID_ARGUMENT;
            }
            let owner = match Self::owner(module) {
                Ok(owner) => owner,
                Err(status) => return status,
            };
            let Some(adapter) = WorldRenderAdapter::get() else {
                return AmmlStatus::NOT_AVAILABLE;
            };
            // SAFETY: `slot_size` is non-null and the caller promises it is readable for the call.
            let slot_size = unsafe { slot_size.read_unaligned() };
            match adapter.group_begin(owner, frame, slot_size) {
                Ok(group) => {
                    // SAFETY: `out` is non-null and the caller promises it is writable.
                    unsafe { out.write_unaligned(group) };
                    AmmlStatus::OK
                }
                Err(status) => status,
            }
        })
    }

    unsafe extern "C" fn mesher_group_append(
        module: AmmlStr,
        frame: u64,
        group: AmmlMeshGroup,
        grid: *const AmmlBlockGrid,
        offset: *const AmmlBlockPos,
        options: *const AmmlMesherOptions,
    ) -> AmmlStatus {
        Boundary::guard(|| {
            if grid.is_null() || offset.is_null() || options.is_null() {
                return AmmlStatus::INVALID_ARGUMENT;
            }
            let owner = match Self::owner(module) {
                Ok(owner) => owner,
                Err(status) => return status,
            };
            let Some(adapter) = WorldRenderAdapter::get() else {
                return AmmlStatus::NOT_AVAILABLE;
            };
            // SAFETY: `grid`, `offset` and `options` are non-null and the caller promises they are readable for the call.
            let (grid, offset, options) =
                unsafe { (grid.read_unaligned(), offset.read_unaligned(), options.read_unaligned()) };
            // The frame, the thread and the dims are checked before a cell is read.
            if let Err(status) = adapter.check_mesher(frame, grid.dims, grid.cell_count as usize) {
                return status;
            }
            let (cells, palette) = match Self::grid_slices(&grid) {
                Ok(slices) => slices,
                Err(status) => return status,
            };
            let origin = [grid.origin.x, grid.origin.y, grid.origin.z];
            let offset = [offset.x, offset.y, offset.z];
            match adapter.group_append(owner, frame, group, (grid.dims, origin, cells), palette, offset, &options) {
                Ok(()) => AmmlStatus::OK,
                Err(status) => status,
            }
        })
    }

    unsafe extern "C" fn mesher_group_finish(
        module: AmmlStr,
        frame: u64,
        group: AmmlMeshGroup,
        out: *mut AmmlMeshBuilt,
    ) -> AmmlStatus {
        Boundary::guard(|| {
            if out.is_null() {
                return AmmlStatus::INVALID_ARGUMENT;
            }
            let owner = match Self::owner(module) {
                Ok(owner) => owner,
                Err(status) => return status,
            };
            let Some(adapter) = WorldRenderAdapter::get() else {
                return AmmlStatus::NOT_AVAILABLE;
            };
            match adapter.group_finish(owner, frame, group) {
                Ok(built) => {
                    // SAFETY: `out` is non-null and the caller promises it is writable.
                    unsafe { out.write_unaligned(built) };
                    AmmlStatus::OK
                }
                Err(status) => status,
            }
        })
    }

    unsafe extern "C" fn mesher_group_discard(module: AmmlStr, group: AmmlMeshGroup) -> AmmlStatus {
        Boundary::guard(|| {
            let owner = match Self::owner(module) {
                Ok(owner) => owner,
                Err(status) => return status,
            };
            match WorldRenderAdapter::get()
                .ok_or(AmmlStatus::NOT_AVAILABLE)
                .and_then(|adapter| adapter.group_discard(owner, group))
            {
                Ok(()) => AmmlStatus::OK,
                Err(status) => status,
            }
        })
    }

    // The cells and the palette of `grid`: `INVALID_ARGUMENT` for a null pointer with a count.
    //
    // Safety (of the caller): the grid's cells and palette are readable for their counts for the call; an empty slice is
    // made without reading.
    fn grid_slices(grid: &AmmlBlockGrid) -> Result<(&[u16], &[AmmlBlockHandle]), AmmlStatus> {
        let null_with_count = |pointer_is_null: bool, count: u32| pointer_is_null && count > 0;
        if null_with_count(grid.cells.is_null(), grid.cell_count)
            || null_with_count(grid.palette.is_null(), grid.palette_len)
        {
            return Err(AmmlStatus::INVALID_ARGUMENT);
        }
        // SAFETY: the caller promises `cell_count` cells at `cells` and `palette_len` handles at `palette`, readable for
        // the call; an empty slice is made without reading.
        Ok(unsafe {
            (
                if grid.cell_count == 0 {
                    &[][..]
                } else {
                    std::slice::from_raw_parts(grid.cells, grid.cell_count as usize)
                },
                if grid.palette_len == 0 {
                    &[][..]
                } else {
                    std::slice::from_raw_parts(grid.palette, grid.palette_len as usize)
                },
            )
        })
    }

    unsafe extern "C" fn block_entity_create(
        module: AmmlStr,
        block: AmmlBlockHandle,
        nbt: *const u8,
        nbt_len: usize,
        options: *const AmmlGhostEntityOptions,
        out: *mut AmmlBlockEntityGhost,
    ) -> AmmlStatus {
        Boundary::guard(|| {
            if out.is_null() || options.is_null() || (nbt.is_null() && nbt_len != 0) {
                return AmmlStatus::INVALID_ARGUMENT;
            }
            let owner = match Self::owner(module) {
                Ok(owner) => owner,
                Err(status) => return status,
            };
            // SAFETY: `options` is non-null and the caller promises it is readable for the call.
            let options = unsafe { options.read_unaligned() };
            // SAFETY: the caller promises `nbt_len` bytes readable at `nbt` for the call; an empty slice is made without
            // reading.
            let nbt = if nbt_len == 0 { &[][..] } else { unsafe { std::slice::from_raw_parts(nbt, nbt_len) } };
            match BlockEntityAdapter::get()
                .ok_or(AmmlStatus::NOT_AVAILABLE)
                .and_then(|adapter| adapter.create(owner, block, nbt, &options))
            {
                Ok(ghost) => {
                    // SAFETY: `out` is non-null and the caller promises it is writable.
                    unsafe { out.write(ghost) };
                    AmmlStatus::OK
                }
                Err(status) => status,
            }
        })
    }

    unsafe extern "C" fn block_entity_free(module: AmmlStr, ghost: AmmlBlockEntityGhost) -> AmmlStatus {
        Boundary::guard(|| {
            let owner = match Self::owner(module) {
                Ok(owner) => owner,
                Err(status) => return status,
            };
            match BlockEntityAdapter::get().ok_or(AmmlStatus::NOT_AVAILABLE).and_then(|a| a.free(owner, ghost)) {
                Ok(()) => AmmlStatus::OK,
                Err(status) => status,
            }
        })
    }

    unsafe extern "C" fn block_entity_draw(
        module: AmmlStr,
        frame: u64,
        ghost: AmmlBlockEntityGhost,
        pos: *const AmmlBlockPos,
    ) -> AmmlStatus {
        Boundary::guard(|| {
            if pos.is_null() {
                return AmmlStatus::INVALID_ARGUMENT;
            }
            let owner = match Self::owner(module) {
                Ok(owner) => owner,
                Err(status) => return status,
            };
            // SAFETY: `pos` is non-null and the caller promises it is readable for the call.
            let pos = unsafe { pos.read_unaligned() };
            match BlockEntityAdapter::get()
                .ok_or(AmmlStatus::NOT_AVAILABLE)
                .and_then(|adapter| adapter.draw(owner, frame, ghost, [pos.x, pos.y, pos.z]))
            {
                Ok(()) => AmmlStatus::OK,
                Err(status) => status,
            }
        })
    }

    // The index of the live mod `module`.
    fn owner(module: AmmlStr) -> Result<usize, AmmlStatus> {
        let state = LoaderState::get().ok_or(AmmlStatus::FAILED)?;
        // SAFETY: the caller promises `module` is valid for the call.
        let module = unsafe { module.as_str() }?;
        state.live_mod(module)
    }
}
