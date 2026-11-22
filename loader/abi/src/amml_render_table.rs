use crate::{
    AmmlBlockEntityGhost, AmmlBlockGrid, AmmlBlockHandle, AmmlBlockPos, AmmlColorQuad, AmmlGhostEntityOptions,
    AmmlMeshBuilt, AmmlMeshGroup, AmmlMeshId, AmmlMeshStyle, AmmlMesherOptions, AmmlStatus, AmmlStr, AmmlTransform,
    AmmlVec3,
};

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
/// `NOT_READY` while the loader's white texture (or, for a block mesh, the renderer's terrain atlas) is not loaded yet,
/// `FAILED` when the frame has no camera or material. A block mesh (one the block mesher built) is drawn with the
/// client's blend-block material and a copy of the renderer's terrain atlas. Area `render.world_canvas` version 2.
pub type RenderMeshDrawFn =
    unsafe extern "C" fn(module: AmmlStr, frame: u64, mesh: AmmlMeshId, transform: *const AmmlTransform) -> AmmlStatus;

/// Gives the clip matrix of the world camera of frame `frame` in `out`: the top of the camera's projection stack times
/// the top of its view stack, relative to the camera, stored by columns (`out[column][row]`, as glm stores it). `frame`
/// is the `frame_id` of the `WorldRendered` event being dispatched on the calling thread (`REFUSED` for any other value
/// and for a call outside the dispatch); `INVALID_ARGUMENT` for a null `out`; `FAILED` when the frame has no camera or a
/// matrix stack is empty. Area `render.world_canvas` version 3.
pub type RenderCanvasClipMatrixFn = unsafe extern "C" fn(frame: u64, out: *mut [[f32; 4]; 4]) -> AmmlStatus;

/// Builds the grid `grid` with the game's block tessellator into a block mesh of the mod `module` (drawn with
/// `mesh_draw` like every mesh) and fills `out`. Only inside the dispatch of frame `frame` on the game thread (else
/// `REFUSED`). Each inner cell is tessellated in every render layer its block needs; the border cells and the empty cells
/// are only neighbours. With `options.skip_matching_world` a cell whose world block is the same game block is not drawn:
/// the first `skipped_cap` of their cell indices are written to `skipped` (null when `skipped_cap` is 0) and
/// `out.skipped_count` counts them all. A build that draws nothing gives `OK` with mesh id 0 and no quads. `REFUSED` for
/// a side above 20 (the game's cache window); `INVALID_ARGUMENT` for a null pointer, `cell_count` other than the product
/// of `dims` or a cell index past the palette; `STALE` for a palette handle that is stale; `FAILED` when the game made no
/// tessellator or mesh. Nothing is tessellated when a check fails. Area `render.block_mesher`.
pub type RenderMesherBuildFn = unsafe extern "C" fn(
    module: AmmlStr,
    frame: u64,
    grid: *const AmmlBlockGrid,
    options: *const AmmlMesherOptions,
    out: *mut AmmlMeshBuilt,
    skipped: *mut u32,
    skipped_cap: u32,
) -> AmmlStatus;

/// Opens a mesh group whose slots are `slot_size` blocks, for the mod `module`, in the dispatch of frame `frame`, and
/// gives it in `out`. A group is a game `MeshData` buffer the loader owns, bound to the mod and the world and usable in any
/// `WorldRendered` dispatch of that world; the loader drops it at the world leave, a render device reset or surface loss and
/// the mod's disable (`STALE` for any later use). `REFUSED` outside the dispatch of `frame`, off the game thread, for a
/// 65th open group of the mod (the loader checks the slot size and does not use it otherwise); `INVALID_ARGUMENT` for a null pointer or a slot side of 0.
/// Area `render.block_mesher` (group calls).
pub type RenderMesherGroupBeginFn = unsafe extern "C" fn(
    module: AmmlStr,
    frame: u64,
    slot_size: *const [u32; 3],
    out: *mut AmmlMeshGroup,
) -> AmmlStatus;

/// Appends the grid `grid`, moved by `offset`, to the open group `group` of the mod `module`, in the dispatch of frame
/// `frame`. The cells are tessellated as `mesher_build` does, at their world positions; a cell at the world position `p`
/// lands at `p - grid.origin + offset` in the group. With `options.skip_matching_world` the cells whose world block matches
/// are left out (their indices are not reported). `STALE` for a group that is gone or a palette handle that is stale,
/// `REFUSED` for a group of another mod or one that would hold more than 65,536 quads (the group is as it was),
/// `INVALID_ARGUMENT` as `mesher_build`, `FAILED` when the game made no object (the group is then dropped). Area
/// `render.block_mesher` (group calls).
pub type RenderMesherGroupAppendFn = unsafe extern "C" fn(
    module: AmmlStr,
    frame: u64,
    group: AmmlMeshGroup,
    grid: *const AmmlBlockGrid,
    offset: *const AmmlBlockPos,
    options: *const AmmlMesherOptions,
) -> AmmlStatus;

/// Ends the group `group` of the mod `module` into one mesh, in the dispatch of frame `frame`, and fills `out`; the
/// group is gone after the call, whatever it gives. The mesh is a block mesh of the mod, as `mesher_build` gives; a group
/// that holds nothing gives `OK` with mesh id 0 and no quads. `out.skipped_count` is 0. `STALE` for a group that is gone,
/// `REFUSED` for a group of another mod, `FAILED` when the game made no mesh. Area `render.block_mesher` (group calls).
pub type RenderMesherGroupFinishFn =
    unsafe extern "C" fn(module: AmmlStr, frame: u64, group: AmmlMeshGroup, out: *mut AmmlMeshBuilt) -> AmmlStatus;

/// Makes a block entity ghost of the block `block` (a handle of the mod `module`) and gives it in `out`: the game's block
/// actor of that block, loaded from the `nbt_len` bytes at `nbt` when `nbt_len` is not 0 (a block entity compound in
/// little-endian NBT, as a structure file stores it), else made with its defaults. With `options.has_chest_pair` the
/// actor is made the lead of a large chest whose other half is at `options.chest_pair` from it, when the game says it is a
/// chest. Only on the game thread with a world joined (`REFUSED` off the game thread, `NOT_READY` without a world).
/// With `options.has_skull` the ghost is drawn as a skull with `options.skull` when the game says it is one.
/// `INVALID_ARGUMENT` for a null pointer, a chest pair that is not one block along x or z, a chest pair on a block that is
/// not a chest, a skull pose with a face above 5 or a turn above 15 or on a block that is not a skull, or a reserved byte
/// that is not 0; `STALE` for a stale block handle; `NOT_AVAILABLE` for a chest pair without area `render.chest_pair` or a skull pose
/// without area `render.skull_pose`; `REFUSED` for a 257th live ghost of the mod; `FAILED` when the game made no actor (a
/// block without a block entity, NBT the game cannot read). Area `render.block_entities`.
pub type RenderBlockEntityCreateFn = unsafe extern "C" fn(
    module: AmmlStr,
    block: AmmlBlockHandle,
    nbt: *const u8,
    nbt_len: usize,
    options: *const AmmlGhostEntityOptions,
    out: *mut AmmlBlockEntityGhost,
) -> AmmlStatus;

/// Frees the ghost `ghost` of the mod `module`. The ghost is gone for the mod at once (`STALE` for any later use); the
/// game actor is released on the `renderLevel` thread at its next call, from whatever thread this call came. `STALE` for a
/// ghost that is gone, `REFUSED` for a ghost of another mod. Area `render.block_entities`.
pub type RenderBlockEntityFreeFn = unsafe extern "C" fn(module: AmmlStr, ghost: AmmlBlockEntityGhost) -> AmmlStatus;

/// Draws the ghost `ghost` of the mod `module` at the world position `pos` with the game's block actor renderer, in the
/// block entity pass `frame` (the `frame_id` of the `BlockEntityPass` being dispatched on the calling thread; `REFUSED`
/// for any other value and outside the dispatch). `STALE` for a ghost that is gone, `REFUSED` for a ghost of another mod,
/// `INVALID_ARGUMENT` for a null `pos`, `FAILED` when the actor has no render component or the pass has no dispatcher.
/// Area `render.block_entities`.
pub type RenderBlockEntityDrawFn = unsafe extern "C" fn(
    module: AmmlStr,
    frame: u64,
    ghost: AmmlBlockEntityGhost,
    pos: *const AmmlBlockPos,
) -> AmmlStatus;

/// Ends the open group `group` of the mod `module` without a mesh: the group is gone for the mod at once (`STALE` for
/// any later use) and its slot is free; its game `MeshData` buffer is destroyed on the `renderLevel` thread at its next
/// call, from whatever thread this call came. For a group the mod drops without `mesher_group_finish`. `STALE` for a
/// group that is gone (finished, discarded, dropped by the loader or of an earlier world), `REFUSED` for a group of
/// another mod. Area `render.block_mesher` (group calls), render table version 6.
pub type RenderMesherGroupDiscardFn = unsafe extern "C" fn(module: AmmlStr, group: AmmlMeshGroup) -> AmmlStatus;

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
    pub canvas_clip_matrix: Option<RenderCanvasClipMatrixFn>,
    pub mesher_build: Option<RenderMesherBuildFn>,
    pub mesher_group_begin: Option<RenderMesherGroupBeginFn>,
    pub mesher_group_append: Option<RenderMesherGroupAppendFn>,
    pub mesher_group_finish: Option<RenderMesherGroupFinishFn>,
    pub block_entity_create: Option<RenderBlockEntityCreateFn>,
    pub block_entity_free: Option<RenderBlockEntityFreeFn>,
    pub block_entity_draw: Option<RenderBlockEntityDrawFn>,
    pub mesher_group_discard: Option<RenderMesherGroupDiscardFn>,
}

impl AmmlRenderTable {
    /// `size` of a v1 render table. A literal, so it keeps its meaning when a later version adds fields.
    pub const SIZE_V1: u32 = 24;
    /// `size` of a v2 render table: v1 plus `mesh_build` and `mesh_draw`.
    pub const SIZE_V2: u32 = 40;
    /// `size` of a v3 render table: v2 plus `canvas_clip_matrix`.
    pub const SIZE_V3: u32 = 48;
    /// `size` of a v4 render table: v3 plus `mesher_build` and the three group calls.
    pub const SIZE_V4: u32 = 80;
    /// `size` of a v5 render table: v4 plus the three block entity ghost calls.
    pub const SIZE_V5: u32 = 104;
    /// `size` of a v6 render table: v5 plus `mesher_group_discard`.
    pub const SIZE_V6: u32 = 112;

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

const _: () = assert!(core::mem::size_of::<AmmlRenderTable>() == AmmlRenderTable::SIZE_V6 as usize);
