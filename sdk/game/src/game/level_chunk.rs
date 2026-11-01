use std::ffi::c_void;
use std::ptr::NonNull;

use crate::game::{
    Block, ChunkBlockPos, ChunkState, Dimension, LevelChunkLayout2651, LevelChunkLayout2660, LevelChunkLayoutKind,
    SubChunk, SubChunkLayout,
};
use crate::{EntryId, GameEntries};

// The address of `$field` in the `LevelChunk` mirror the `LevelChunk.mSubChunks` entry selects, as a raw pointer of
// the field type (no Rust reference to game memory is made); `None` when no mirror is selected. The callers read
// it under their own SAFETY comment.
macro_rules! field {
    ($chunk:expr, $field:ident) => {{
        let ptr = $chunk.ptr.as_ptr();
        // SAFETY: `from_raw` guarantees a live LevelChunk of the build the entries were resolved for, and the
        // selected mirror is that build's layout, so the field place is inside the object. Only its address is
        // taken.
        $chunk.layout().map(|layout| unsafe {
            match layout {
                LevelChunkLayoutKind::Build2651 => &raw const (*ptr.cast::<LevelChunkLayout2651>()).$field,
                LevelChunkLayoutKind::Build2660 => &raw const (*ptr.cast::<LevelChunkLayout2660>()).$field,
            }
        })
    }};
}

/// A `LevelChunk*` of the game.
///
/// Every read here is a plain memory read through the `LevelChunk` mirror of the build, or a call of game code. All of it is for the game
/// thread: the game changes a chunk on that thread and nothing here takes a lock.
pub struct LevelChunk<'a> {
    ptr: NonNull<c_void>,
    entries: &'a GameEntries,
}

impl<'a> LevelChunk<'a> {
    /// Every entry the reads here use. `LevelChunk.mSubChunks` selects the mirror, so `position`, `min_y`,
    /// `max_y`, `loaded`, `dimension` and the heightmaps need it alone; the sub-chunks also need `SubChunk.size`
    /// and `SubChunk.isPlaceHolderSubChunk`, and `block` needs `LevelChunk.getBlock`.
    pub const ENTRIES: [EntryId; 4] = [
        EntryId::LEVEL_CHUNK_SUB_CHUNKS,
        EntryId::SUB_CHUNK_SIZE,
        EntryId::SUB_CHUNK_IS_PLACE_HOLDER_SUB_CHUNK,
        EntryId::LEVEL_CHUNK_GET_BLOCK,
    ];

    /// The number of entries in a heightmap: one per column of 16 x 16 blocks.
    pub const HEIGHTMAP_LEN: usize = 256;

    /// Wraps `ptr`; `None` for a null pointer.
    ///
    /// # Safety
    /// `ptr` must be null or a live `LevelChunk` of the game build `entries` was resolved for, and must stay live
    /// while the wrapper is used.
    #[allow(unsafe_code)]
    pub unsafe fn from_raw(ptr: *mut c_void, entries: &'a GameEntries) -> Option<Self> {
        NonNull::new(ptr).map(|ptr| Self { ptr, entries })
    }

    /// The wrapped pointer.
    pub fn as_ptr(&self) -> *mut c_void {
        self.ptr.as_ptr()
    }

    // The mirror of this game build; `None` when the `LevelChunk.mSubChunks` entry is off or matches no mirror.
    fn layout(&self) -> Option<LevelChunkLayoutKind> {
        LevelChunkLayoutKind::of(self.entries)
    }

    /// The chunk position (x, z) in chunks. `None` when no mirror is selected. Game thread.
    #[allow(unsafe_code)]
    pub fn position(&self) -> Option<[i32; 2]> {
        let at = field!(self, position)?;
        // SAFETY: `from_raw` guarantees a live LevelChunk; `at` is its `mPosition`, a `ChunkPos` of two ints.
        Some(unsafe { at.read_unaligned() })
    }

    /// The lowest block y of the chunk (`mMin.y`). `None` when no mirror is selected. Game thread.
    #[allow(unsafe_code)]
    pub fn min_y(&self) -> Option<i32> {
        let at = field!(self, min)?;
        // SAFETY: `from_raw` guarantees a live LevelChunk; `at` is its `mMin`, a `BlockPos` with y second.
        Some(unsafe { at.read_unaligned() }[1])
    }

    /// The highest block y of the chunk (`mMax.y`). `None` when no mirror is selected. Game thread.
    #[allow(unsafe_code)]
    pub fn max_y(&self) -> Option<i32> {
        let at = field!(self, max)?;
        // SAFETY: as `min_y`, for `mMax`.
        Some(unsafe { at.read_unaligned() }[1])
    }

    /// Whether the chunk is fully loaded (`mLoadState` is `ChunkState::LOADED`); `false` when no mirror is
    /// selected. Game thread.
    #[allow(unsafe_code)]
    pub fn loaded(&self) -> bool {
        let Some(at) = field!(self, load_state) else {
            return false;
        };
        // SAFETY: `from_raw` guarantees a live LevelChunk; `at` is its `mLoadState`, an atomic byte.
        unsafe { at.read_volatile() == ChunkState::LOADED }
    }

    /// The dimension of the chunk. `None` when no mirror is selected or the pointer is null. Game thread.
    #[allow(unsafe_code)]
    pub fn dimension(&self) -> Option<Dimension<'a>> {
        let at = field!(self, dimension)?;
        // SAFETY: `from_raw` guarantees a live LevelChunk; `at` is its `mDimension`, a reference stored as a
        // pointer.
        let dimension = unsafe { at.read_unaligned() };
        // SAFETY: the pointer is the chunk's live Dimension, or null.
        unsafe { Dimension::from_raw(dimension, self.entries) }
    }

    // The height at `index` of the heightmap at `map`; `None` for `index >= 256`.
    #[allow(unsafe_code)]
    fn height_at(map: *const [i16; 256], index: usize) -> Option<i16> {
        if index >= Self::HEIGHTMAP_LEN {
            return None;
        }
        // SAFETY: `map` is a 256-entry heightmap of a live LevelChunk and `index` is below 256.
        Some(unsafe { map.cast::<i16>().add(index).read_unaligned() })
    }

    /// The height of the column `index` (`(z << 4) | x`) from `mHeightmap`. `None` for `index >= 256` or when no
    /// mirror is selected. Game thread.
    #[allow(unsafe_code)]
    pub fn height(&self, index: usize) -> Option<i16> {
        Self::height_at(field!(self, heightmap)?, index)
    }

    /// As [`LevelChunk::height`], from `mRenderHeightmap`.
    #[allow(unsafe_code)]
    pub fn render_height(&self, index: usize) -> Option<i16> {
        Self::height_at(field!(self, render_heightmap)?, index)
    }

    // The begin address and the count of `mSubChunks`. `None` when no mirror is selected, `SubChunk.size` is not
    // the size of the sub-chunk mirror, the vector has no storage, or its pointers are not consistent.
    #[allow(unsafe_code)]
    fn sub_chunk_span(&self) -> Option<(usize, usize)> {
        let size = size_of::<SubChunkLayout>();
        if self.entries.get(EntryId::SUB_CHUNK_SIZE)? != u64::try_from(size).ok()? {
            return None;
        }
        // An MSVC vector is three pointers: begin, end and capacity.
        let at = field!(self, sub_chunks)?.cast::<usize>();
        // SAFETY: `from_raw` guarantees a live LevelChunk; `at` is its `mSubChunks`.
        let (begin, end) = unsafe { (at.read_unaligned(), at.add(1).read_unaligned()) };
        let bytes = end.checked_sub(begin)?;
        (begin != 0).then_some((begin, bytes / size))
    }

    /// The number of sub-chunks; 0 when an entry is off. Game thread.
    pub fn sub_chunk_count(&self) -> usize {
        self.sub_chunk_span().map_or(0, |(_, count)| count)
    }

    /// The sub-chunk `i`, counted from the bottom. `None` for `i >= sub_chunk_count()` or an entry that is off.
    /// The wrapper points into the game's vector, so it must not be kept past the next change of the chunk. Game
    /// thread.
    #[allow(unsafe_code)]
    pub fn sub_chunk(&self, i: usize) -> Option<SubChunk<'a>> {
        let (begin, count) = self.sub_chunk_span()?;
        if i >= count {
            return None;
        }
        let at = begin.checked_add(i.checked_mul(size_of::<SubChunkLayout>())?)?;
        // SAFETY: `i < count`, so the element is inside the vector of a live chunk.
        unsafe { SubChunk::from_raw(at as *mut c_void, self.entries) }
    }

    /// The block at `pos`. `None` when an entry is off or the game returns null. Game thread.
    #[allow(unsafe_code)]
    pub fn block(&self, pos: ChunkBlockPos) -> Option<Block<'a>> {
        let function = usize::try_from(self.entries.get(EntryId::LEVEL_CHUNK_GET_BLOCK)?).ok()?;
        if function == 0 {
            return None;
        }
        // SAFETY: the entry is the address of `Block const& LevelChunk::getBlock(ChunkBlockPos const&) const`, a
        // non-static member that returns a reference, which is a pointer in RAX; `ChunkBlockPos` is the 4-byte
        // `#[repr(C)]` mirror of the game struct.
        let call: unsafe extern "C" fn(*mut c_void, *const ChunkBlockPos) -> *mut c_void =
            unsafe { std::mem::transmute(function as *const c_void) };
        // SAFETY: the signature is as above, `from_raw` guarantees a live LevelChunk and `pos` lives across the
        // call.
        let block = unsafe { call(self.ptr.as_ptr(), &raw const pos) };
        // SAFETY: the game returns a live Block reference, or null.
        unsafe { Block::from_raw(block, self.entries) }
    }
}
