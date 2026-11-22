use std::ffi::c_void;
use std::marker::PhantomData;

use amml_abi::{
    AmmlAabb, AmmlActorBox, AmmlBlockHit, AmmlBlockPos, AmmlKindsResult, AmmlRayOptions, AmmlSurfaceSample,
    AmmlWorldTable,
};

use crate::{
    Aabb, ActorBox, ApiError, Block, BlockHit, BlockPos, BlockTypeId, Blocks, ColumnPos, KindsInBox, LevelChunk,
    RayOptions, SurfaceSample, Vec3,
};

/// The block source of the dimension of a `LocalPlayer` (`LocalPlayer::block_source`), as an opaque handle. Two values
/// are equal when they name the same block source. It lives no longer than the event that gave its player; the loader
/// checks on every call that it is still the current block source of the local player of the primary client, and gives
/// a refusal (`None` or an error) for a handle kept past its frame, past a dimension change or past a world leave.
pub struct BlockSource<'f> {
    handle: *const c_void,
    world: Option<AmmlWorldTable>,
    module: &'static str,
    _event: PhantomData<&'f ()>,
}

impl<'f> BlockSource<'f> {
    // The surfaces one loader call gives at most; a column with more takes one more call for each batch.
    const SCAN_BATCH: usize = 32;
    /// The most boxes `actors_in` gives.
    pub const MAX_ACTORS: usize = 256;
    /// The most cells a box of `kinds_in_box` may hold.
    pub const KINDS_CELL_CAP: u32 = 65536;

    pub(crate) fn new(handle: *const c_void, world: Option<AmmlWorldTable>, module: &'static str) -> Self {
        Self { handle, world, module, _event: PhantomData }
    }

    /// The block at `pos`. `None` when the game has no block there, when the loader refuses the call (the block source
    /// is from an earlier frame, world or dimension, or the call is not on the game thread) or does not serve the
    /// area. The block is valid in this game frame only. Area `world.blocks` (`Area::WORLD_BLOCKS`).
    pub fn block(&self, pos: BlockPos) -> Option<Block<'f>> {
        let world = self.world?;
        let read = world.block_at?;
        let at = AmmlBlockPos::from(pos);
        let mut out: *const c_void = std::ptr::null();
        // SAFETY: the loader checks the handle before it reads game memory; `at` and `out` are live locals.
        #[allow(unsafe_code)]
        let status = unsafe { read(self.handle, &raw const at, &mut out) };
        (status.is_ok() && !out.is_null()).then(|| Block::new(out, self.handle, pos, world, self.module))
    }

    /// The chunk at chunk position (`x`, `z`), in chunks (a block x or z divided by 16, rounded down). `None` when the
    /// game has no chunk there, when the loader refuses the call (the block source is from an earlier world or
    /// dimension, for example) or does not serve the area. The chunk is valid in this game frame only. Area
    /// `world.chunk_read` (`Area::WORLD_CHUNK_READ`).
    pub fn chunk(&self, x: i32, z: i32) -> Option<LevelChunk<'f>> {
        let world = self.world?;
        let read = world.chunk?;
        let mut out: *const c_void = std::ptr::null();
        // SAFETY: the loader checks the handle before it reads game memory, and `out` is a live local.
        #[allow(unsafe_code)]
        let status = unsafe { read(self.handle, x, z, &mut out) };
        (status.is_ok() && !out.is_null()).then(|| LevelChunk::new(out, world))
    }

    /// The distinct block types in the box from `min` to `max` (both included), by full name, in the order the loader
    /// meets them (x, then z, then y, each upward). Air counts as a kind. At most `max_kinds` are listed.
    ///
    /// Every chunk of the box is checked loaded before a cell is read: a chunk that is not, or a block that cannot be
    /// read, gives `Ok(KindsInBox::NotLoaded)`; more kinds than `max_kinds` gives `Ok(KindsInBox::TooMany)`; neither
    /// lists anything. A box with `min` above `max` on an axis, or with more than `KINDS_CELL_CAP` (65536) cells, gives
    /// `Err(Refused)`, as do a block source that is from an earlier frame, world or dimension and a call off the game
    /// thread. `Err(NotAvailable)` without the area. Area `world.kinds_in_box` (`Area::WORLD_KINDS_IN_BOX`), which needs
    /// `world.chunk_read` for the names.
    pub fn kinds_in_box(&self, min: BlockPos, max: BlockPos, max_kinds: u32) -> Result<KindsInBox, ApiError> {
        let scan = self.world.and_then(|world| world.kinds_in_box).ok_or(ApiError::NotAvailable)?;
        // A box holds at most `KINDS_CELL_CAP` cells, so no more kinds than that can be found.
        let room = max_kinds.min(Self::KINDS_CELL_CAP);
        let mut ids = vec![0u32; room as usize];
        let mut result = AmmlKindsResult::default();
        let (min, max) = (AmmlBlockPos::from(min), AmmlBlockPos::from(max));
        // SAFETY: the loader checks the handle before it reads game memory; `min`, `max` and `result` are live locals
        // and `ids` holds `room` entries.
        #[allow(unsafe_code)]
        let status =
            unsafe { scan(self.handle, &raw const min, &raw const max, ids.as_mut_ptr(), room, &raw mut result) };
        ApiError::check(status)?;
        match result.kind {
            AmmlKindsResult::KINDS => {
                let blocks = Blocks::new(self.world, self.module);
                let count = (result.count as usize).min(ids.len());
                ids[..count]
                    .iter()
                    .map(|id| blocks.type_name(BlockTypeId(*id)))
                    .collect::<Option<Vec<String>>>()
                    .map(KindsInBox::Kinds)
                    .ok_or(ApiError::Loader(amml_abi::AmmlStatus::FAILED))
            }
            AmmlKindsResult::NOT_LOADED => Ok(KindsInBox::NotLoaded),
            AmmlKindsResult::TOO_MANY => Ok(KindsInBox::TooMany),
            _ => Err(ApiError::Loader(amml_abi::AmmlStatus::FAILED)),
        }
    }

    /// Finds where a mob can stand in `column`, for the block ys `y_min` to `y_max` (both included), and writes the
    /// surfaces to `out`, lowest first, at most `out.len()` of them. Gives the number written.
    ///
    /// A surface is a block that gives full support on its top face, with air above it or a block that has an empty
    /// collision shape (a flower, a torch, short grass). The light of a surface is the raw sky and block light of the
    /// block above it, with no sky darken. The scan reads nothing below the lowest block of the chunk and nothing above
    /// its highest, so the highest surface is one below the highest block of the chunk.
    ///
    /// `None` when the chunk of the column is not loaded (or its top is still a placeholder), when a block of the column
    /// was not readable (the client has not received it yet), when the loader refuses the call (the block source is from
    /// an earlier frame, world or dimension, or the call is not on the game thread) or does not serve the area. Area
    /// `world.scan_surfaces` (`Area::WORLD_SCAN_SURFACES`), which needs `world.chunk_read`.
    pub fn scan_surfaces(&self, column: ColumnPos, y_min: i32, y_max: i32, out: &mut [SurfaceSample]) -> Option<usize> {
        let scan = self.world?.scan_surfaces?;
        let mut batch = [AmmlSurfaceSample::default(); Self::SCAN_BATCH];
        let mut filled = 0usize;
        let mut from = y_min;
        loop {
            let room = (out.len() - filled).min(Self::SCAN_BATCH);
            let mut count = 0u32;
            // SAFETY: the loader checks the handle before it reads game memory; `batch` holds at least `room` entries
            // (`room` is at most its length) and `count` is a live local.
            #[allow(unsafe_code)]
            let status = unsafe {
                scan(
                    self.handle,
                    column.x,
                    column.z,
                    from,
                    y_max,
                    batch.as_mut_ptr(),
                    u32::try_from(room).ok()?,
                    &mut count,
                )
            };
            if !status.is_ok() {
                return None;
            }
            let found = usize::try_from(count).ok()?.min(room);
            for (slot, sample) in out[filled..].iter_mut().zip(&batch[..found]) {
                *slot = SurfaceSample::from(*sample);
            }
            filled += found;
            // A batch that is not full holds every surface left; a full one is followed by a batch above its last
            // surface, until `out` is full.
            if found < room || found == 0 || filled == out.len() {
                break;
            }
            match batch[found - 1].pos.y.checked_add(1) {
                Some(next) if next <= y_max => from = next,
                _ => break,
            }
        }
        Some(filled)
    }

    /// The first block the ray from `from` to `to` meets, with the point where it met it and the face it entered
    /// through. `None` when the ray meets no block within `options.max_steps` blocks, when a point is not finite, when
    /// the loader refuses the call (the block source is from an earlier frame, world or dimension, or the call is not on
    /// the game thread) or does not serve the area. Area `world.raycast` (`Area::WORLD_RAYCAST`).
    pub fn raycast(&self, from: Vec3, to: Vec3, options: RayOptions) -> Option<BlockHit> {
        let raycast = self.world?.raycast?;
        let mut out = AmmlBlockHit::default();
        // SAFETY: the loader checks the handle before it reads game memory, and `out` is a live local.
        #[allow(unsafe_code)]
        let status = unsafe { raycast(self.handle, from.into(), to.into(), AmmlRayOptions::from(options), &mut out) };
        (status.is_ok() && out.hit != 0).then(|| BlockHit::from(out))
    }

    /// The boxes of the actors the game keeps here whose boxes meet `aabb`, at most `MAX_ACTORS` (256), each a copy
    /// read in the call; the local player is marked. Empty when there are none, when the box is not finite, when the
    /// loader refuses the call or does not serve the area. Area `world.actors_in` (`Area::WORLD_ACTORS_IN`).
    pub fn actors_in(&self, aabb: Aabb) -> Vec<ActorBox> {
        let Some(actors_in) = self.world.and_then(|world| world.actors_in) else { return Vec::new() };
        let mut out = vec![AmmlActorBox::default(); Self::MAX_ACTORS];
        let mut count = 0u32;
        // SAFETY: the loader checks the handle before it reads game memory; `out` holds `MAX_ACTORS` boxes and `count`
        // is a live local.
        #[allow(unsafe_code)]
        let status = unsafe {
            actors_in(self.handle, AmmlAabb::from(aabb), out.as_mut_ptr(), Self::MAX_ACTORS as u32, &mut count)
        };
        if !status.is_ok() {
            return Vec::new();
        }
        out.truncate((count as usize).min(Self::MAX_ACTORS));
        out.into_iter().map(ActorBox::from).collect()
    }
}

impl PartialEq for BlockSource<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.handle == other.handle
    }
}
