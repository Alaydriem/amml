use crate::{AmmlColorQuad, AmmlMeshId, AmmlMeshStyle, AmmlStatus, AmmlStr, AmmlTransform, AmmlVec3};

/// Gives the camera position of the world view of frame `frame` in `out`, in world coordinates. `frame` is the
/// `frame_id` of the `WorldRendered` event being dispatched on the calling thread (`REFUSED` for any other value, and
/// for a call outside the dispatch); `FAILED` when no view of the frame belongs to the running level renderer. Area
/// `render.world_canvas`.
pub type RenderCanvasCameraPositionFn = unsafe extern "C" fn(frame: u64, out: *mut AmmlVec3) -> AmmlStatus;

/// Frees the mesh `mesh` of the mod `module`. The mesh is gone for the mod at once (`STALE` for any later use); the game
/// object is destroyed on the render thread at the next `renderLevel` call, from whatever thread this call came.
/// `STALE` for an id of an older generation or an id that was freed, `REFUSED` for a mesh of another mod. Area
/// `render.world_canvas`.
pub type RenderMeshFreeFn = unsafe extern "C" fn(module: AmmlStr, mesh: AmmlMeshId) -> AmmlStatus;

/// Builds a mesh of `count` quads at `quads` in the frame's tessellator for the mod `module` and gives its id in
/// `out`. Only inside the dispatch of frame `frame` (else `REFUSED`). `INVALID_ARGUMENT` for no quads or a null
/// pointer, `REFUSED` for more than 8191 quads, `NOT_AVAILABLE` for a light or texture that this version does not
/// serve, `FAILED` when the game made no mesh. Area `render.world_canvas` version 2.
pub type RenderMeshBuildFn = unsafe extern "C" fn(
    module: AmmlStr,
    frame: u64,
    quads: *const AmmlColorQuad,
    count: u32,
    style: AmmlMeshStyle,
    out: *mut AmmlMeshId,
) -> AmmlStatus;

/// Draws the mesh `mesh` of the mod `module` with `transform` in frame `frame`. Only inside the dispatch of that frame
/// (else `REFUSED`). `STALE` for a freed mesh or one of an older generation, `REFUSED` for a mesh of another mod,
/// `NOT_READY` while the loader's white texture is not loaded yet, `FAILED` when the frame has no camera or material.
/// Area `render.world_canvas` version 2.
pub type RenderMeshDrawFn =
    unsafe extern "C" fn(module: AmmlStr, frame: u64, mesh: AmmlMeshId, transform: *const AmmlTransform) -> AmmlStatus;

/// The render functions of `AmmlServices` (areas `render.*`). The table only grows: a function added later is
/// appended, `size` and `version` grow, and a reader calls a function only when `size` covers it.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct AmmlRenderTable {
    pub size: u32,
    pub version: u32,
    pub canvas_camera_position: RenderCanvasCameraPositionFn,
    pub mesh_free: RenderMeshFreeFn,
    pub mesh_build: Option<RenderMeshBuildFn>,
    pub mesh_draw: Option<RenderMeshDrawFn>,
}

impl AmmlRenderTable {
    /// `size` of a v1 render table. A literal, so it keeps its meaning when a later version adds fields.
    pub const SIZE_V1: u32 = 24;
    /// `size` of a v2 render table: v1 plus `mesh_build` and `mesh_draw`.
    pub const SIZE_V2: u32 = 40;

    /// A copy of the table at `table`, read as far as its own `size` covers; a later field is `None`. A null table, or
    /// one shorter than v1, gives `None`.
    ///
    /// # Safety
    /// A non-null `table` points to a table readable for its own `size` bytes, at least 4.
    #[allow(unsafe_code)]
    pub unsafe fn read(table: *const AmmlRenderTable) -> Option<AmmlRenderTable> {
        if table.is_null() {
            return None;
        }
        // SAFETY: the caller guarantees the leading `size` field is readable.
        let size = unsafe { table.cast::<u32>().read_unaligned() };
        if size < Self::SIZE_V1 {
            return None;
        }
        let covered = (size as usize).min(core::mem::size_of::<AmmlRenderTable>());
        let mut copy = core::mem::MaybeUninit::<AmmlRenderTable>::zeroed();
        // SAFETY: the caller guarantees `covered` bytes are readable at `table`; `copy` has room for the whole
        // struct and the two do not overlap.
        unsafe { core::ptr::copy_nonoverlapping(table.cast::<u8>(), copy.as_mut_ptr().cast::<u8>(), covered) };
        // SAFETY: the copied bytes cover every v1 field, written from a valid table; every later field is an
        // `Option` of a function pointer, for which all zero bytes are `None`.
        Some(unsafe { copy.assume_init() })
    }
}

const _: () = assert!(core::mem::size_of::<AmmlRenderTable>() == AmmlRenderTable::SIZE_V2 as usize);
