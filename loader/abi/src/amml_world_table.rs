use core::ffi::c_void;

use crate::{
    AmmlAabb, AmmlActorBox, AmmlApplyStats, AmmlBlockHit, AmmlChunkLoad, AmmlColumnReadStats, AmmlColumnRequest,
    AmmlColumnSample, AmmlRayOptions, AmmlShapeOp, AmmlStatus, AmmlStr, AmmlSurfaceSample, AmmlVec3,
};

/// Gives the sky darkness of `dimension` in `out` (0 for full daylight, up to 15). `dimension` is a handle that
/// `ActorDimensionFn` gave in the same frame. Area `world.sky_darken`.
pub type WorldSkyDarkenFn = unsafe extern "C" fn(dimension: *const c_void, out: *mut u8) -> AmmlStatus;

/// Gives the chunk at chunk position (`x`, `z`) of `source` in `out`, as a handle for the chunk calls; null when the
/// game has no chunk there. `source` is a block source handle that `ActorBlockSourceFn` gave in the same frame. The
/// chunk handle is valid in the game frame of this call only: an older handle gives `REFUSED`. Area
/// `world.chunk_read`.
pub type WorldChunkFn =
    unsafe extern "C" fn(source: *const c_void, x: i32, z: i32, out: *mut *const c_void) -> AmmlStatus;

/// Gives how much of `chunk` the client has in `out`. Area `world.chunk_read`.
pub type WorldChunkLoadStateFn = unsafe extern "C" fn(chunk: *const c_void, out: *mut AmmlChunkLoad) -> AmmlStatus;

/// Reads the columns of `chunk` that `request` asks for into `out`, which holds `len` samples; `len` must be 256 and
/// column `(x, z)` is entry `z * 16 + x`. Entries of columns that were not asked for are not written. `stats` counts
/// the columns read, skipped and to retry. Area `world.chunk_read`.
pub type WorldReadColumnsFn = unsafe extern "C" fn(
    chunk: *const c_void,
    request: *const AmmlColumnRequest,
    out: *mut AmmlColumnSample,
    len: u32,
    stats: *mut AmmlColumnReadStats,
) -> AmmlStatus;

/// Copies the name of the block type `id` (such as `minecraft:stone`) to `buf` (up to `cap` bytes) and gives its full
/// length in `out_len`; a name longer than `cap` is cut. `INVALID_ARGUMENT` for an id the loader never gave or a name
/// that could not be read. Allowed from any thread. Area `world.chunk_read`.
pub type WorldBlockTypeNameFn =
    unsafe extern "C" fn(id: u32, buf: *mut u8, cap: usize, out_len: *mut usize) -> AmmlStatus;

/// Copies the name of the biome `id` (such as `plains`) to `buf` (up to `cap` bytes) and gives its full length in
/// `out_len`; a name longer than `cap` is cut. The loader reads the name the first time a `read_columns` of the current
/// world meets the id. `INVALID_ARGUMENT` for an id not met yet in the current world or a name that could not be read.
/// Allowed from any thread. Area `world.biomes`.
pub type WorldBiomeNameFn = unsafe extern "C" fn(id: u16, buf: *mut u8, cap: usize, out_len: *mut usize) -> AmmlStatus;

/// Finds where a mob can stand in the column at block position (`x`, `z`) of `source` with a block y from `y_min` to
/// `y_max` (both included), lowest first, and writes at most `len` samples to `out`; `out_count` gets the number
/// written. A surface is a block that gives full support on its top face, with air or a block that has an empty collision
/// shape above it; the light is the raw sky and block light of the block above. The scan reads nothing below the
/// lowest block of the chunk and nothing above its highest, so the highest surface is one below the highest block.
/// `source` is a block source handle that `ActorBlockSourceFn` gave in the same frame. `FAILED` when the chunk is not
/// loaded or a block could not be read, `REFUSED` for a block source that is not the current one and off the game
/// thread. Area `world.scan_surfaces`.
pub type WorldScanSurfacesFn = unsafe extern "C" fn(
    source: *const c_void,
    x: i32,
    z: i32,
    y_min: i32,
    y_max: i32,
    out: *mut AmmlSurfaceSample,
    len: u32,
    out_count: *mut u32,
) -> AmmlStatus;

/// Hands the `count` shape changes at `ops` of the mod `module` to the game in one packet, in order, as shapes of
/// `dimension`, and writes how it went to `out` (may be null). The game draws them until a remove, a world leave or a
/// new client packet handler (`WorldShapesTargetEpochFn`). The mod's shape ids are its own: the loader maps them to game
/// ids. `REFUSED` for more than `WorldShapesMaxBatchFn` ops, off the game thread or from inside a loader delivery;
/// `NOT_READY` before a world is joined and before the loader saw the client's packet handler in this world;
/// `INVALID_ARGUMENT` for an unknown kind or a number that is not finite (then nothing goes out); `FAILED` when the game
/// makes no packet or the game heap is not available. Area `world.shapes`.
pub type WorldShapesApplyFn = unsafe extern "C" fn(
    module: AmmlStr,
    dimension: i32,
    ops: *const AmmlShapeOp,
    count: u32,
    out: *mut AmmlApplyStats,
) -> AmmlStatus;

/// The most ops one `shapes_apply` call takes. Area `world.shapes`.
pub type WorldShapesMaxBatchFn = unsafe extern "C" fn() -> u32;

/// Gives the generation of the client packet handler the loader holds in `out`. When it changes, the game dropped
/// every shape, and a mod sends its shapes again. `NOT_READY` before the loader saw the handler in this world. Allowed
/// from any thread. Area `world.shapes`.
pub type WorldShapesTargetEpochFn = unsafe extern "C" fn(out: *mut u32) -> AmmlStatus;

/// Walks the ray from `from` to `to` through the blocks of `source` and writes the first block it meets to `out` (`hit`
/// 1), or `hit` 0 when it meets none within `options.max_steps` blocks. `OK` in both cases. `source` is a block source
/// handle that `ActorBlockSourceFn` gave in the same frame. `INVALID_ARGUMENT` for a point that is not finite or a null
/// `out`; `REFUSED` for a block source that is not the current one and off the game thread; `NOT_AVAILABLE` for
/// `every_block` 0 without `world.chunk_read`. Area `world.raycast`.
pub type WorldRaycastFn = unsafe extern "C" fn(
    source: *const c_void,
    from: AmmlVec3,
    to: AmmlVec3,
    options: AmmlRayOptions,
    out: *mut AmmlBlockHit,
) -> AmmlStatus;

/// Writes the boxes of the actors the game keeps in `source` whose boxes meet `aabb` to `out`, at most `cap` of them,
/// and their number to `out_count`. Each box is a copy read in the call; the local player of the primary client is
/// marked. `INVALID_ARGUMENT` for a box with a number that is not finite or a null pointer; `REFUSED` as `raycast`;
/// `FAILED` when the game's answer could not be read. Area `world.actors_in`.
pub type WorldActorsInFn = unsafe extern "C" fn(
    source: *const c_void,
    aabb: AmmlAabb,
    out: *mut AmmlActorBox,
    cap: u32,
    out_count: *mut u32,
) -> AmmlStatus;

/// The world functions of `AmmlServices` (areas `world.*`). Every function but `block_type_name`, `biome_name`,
/// `shapes_max_batch` and `shapes_target_epoch` runs on the game thread; a function that takes a handle checks that it
/// is the current handle of the local player of the primary client (`REFUSED` otherwise). The table only grows: a
/// function added later is appended, `size` and `version` grow, and a reader calls a function only when `size` covers
/// it.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct AmmlWorldTable {
    pub size: u32,
    pub version: u32,
    pub sky_darken: WorldSkyDarkenFn,
    pub chunk: Option<WorldChunkFn>,
    pub chunk_load_state: Option<WorldChunkLoadStateFn>,
    pub read_columns: Option<WorldReadColumnsFn>,
    pub block_type_name: Option<WorldBlockTypeNameFn>,
    pub biome_name: Option<WorldBiomeNameFn>,
    pub scan_surfaces: Option<WorldScanSurfacesFn>,
    pub shapes_apply: Option<WorldShapesApplyFn>,
    pub shapes_max_batch: Option<WorldShapesMaxBatchFn>,
    pub shapes_target_epoch: Option<WorldShapesTargetEpochFn>,
    pub raycast: Option<WorldRaycastFn>,
    pub actors_in: Option<WorldActorsInFn>,
}

impl AmmlWorldTable {
    /// `size` of a v1 world table. A literal, so it keeps its meaning when a later version adds fields.
    pub const SIZE_V1: u32 = 16;
    /// `size` of a v2 world table: v1 plus `chunk`, `chunk_load_state`, `read_columns` and `block_type_name`.
    pub const SIZE_V2: u32 = 48;
    /// `size` of a v3 world table: v2 plus `biome_name`.
    pub const SIZE_V3: u32 = 56;
    /// `size` of a v4 world table: v3 plus `scan_surfaces`.
    pub const SIZE_V4: u32 = 64;
    /// `size` of a v5 world table: v4 plus `shapes_apply`, `shapes_max_batch` and `shapes_target_epoch`.
    pub const SIZE_V5: u32 = 88;
    /// `size` of a v6 world table: v5 plus `raycast` and `actors_in`.
    pub const SIZE_V6: u32 = 104;

    /// A copy of the table at `table`, read as far as its own `size` covers; a field a later version adds reads
    /// as zero bytes (`None`). A null table, or one shorter than v1, gives `None`.
    ///
    /// # Safety
    /// A non-null `table` points to a table readable for its own `size` bytes, at least 4.
    #[allow(unsafe_code)]
    pub unsafe fn read(table: *const AmmlWorldTable) -> Option<AmmlWorldTable> {
        if table.is_null() {
            return None;
        }
        // SAFETY: the caller guarantees the leading `size` field is readable.
        let size = unsafe { table.cast::<u32>().read_unaligned() };
        if size < Self::SIZE_V1 {
            return None;
        }
        let covered = (size as usize).min(core::mem::size_of::<AmmlWorldTable>());
        let mut copy = core::mem::MaybeUninit::<AmmlWorldTable>::zeroed();
        // SAFETY: the caller guarantees `covered` bytes are readable at `table`; `copy` has room for the whole
        // struct and the two do not overlap.
        unsafe { core::ptr::copy_nonoverlapping(table.cast::<u8>(), copy.as_mut_ptr().cast::<u8>(), covered) };
        // SAFETY: the copied bytes cover every v1 field, written from a valid table; every later field is an
        // `Option` of a function pointer, for which all zero bytes are `None`.
        Some(unsafe { copy.assume_init() })
    }
}

const _: () = assert!(core::mem::size_of::<AmmlWorldTable>() == AmmlWorldTable::SIZE_V6 as usize);
