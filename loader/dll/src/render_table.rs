use amml_abi::{
    AmmlColorQuad, AmmlMeshId, AmmlMeshStyle, AmmlRenderTable, AmmlStatus, AmmlStr, AmmlTransform, AmmlVec3,
};

use crate::{Boundary, LoaderState, WorldRenderAdapter};

const TABLE_VERSION: u32 = 2;

pub(crate) static RENDER: AmmlRenderTable = AmmlRenderTable {
    size: AmmlRenderTable::SIZE_V2,
    version: TABLE_VERSION,
    canvas_camera_position: RenderTable::canvas_camera_position,
    mesh_free: RenderTable::mesh_free,
    mesh_build: Some(RenderTable::mesh_build),
    mesh_draw: Some(RenderTable::mesh_draw),
};

/// The render table behind `AmmlServices::render`, and its `extern "C"` functions. Each function checks its pointers
/// and delegates to `WorldRenderAdapter` inside the boundary, so a panic never crosses it. With no world render
/// adapter installed (a host without one) every call gives `NOT_AVAILABLE`.
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

    // The index of the live mod `module`.
    fn owner(module: AmmlStr) -> Result<usize, AmmlStatus> {
        let state = LoaderState::get().ok_or(AmmlStatus::FAILED)?;
        // SAFETY: the caller promises `module` is valid for the call.
        let module = unsafe { module.as_str() }?;
        state.live_mod(module)
    }
}
