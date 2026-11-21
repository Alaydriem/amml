use std::collections::HashSet;
use std::ffi::c_void;
use std::sync::Mutex;

use amml_abi::{AmmlChunkLoad, AmmlColumnReadStats, AmmlColumnRequest, AmmlColumnSample, AmmlRgba, AmmlStatus};
use amml_runtime::{ColumnBlocks, ColumnWalk, SurfaceHit, SurfaceRule, WalkBlock, WalkedColumn};

use crate::{
    BiomeReader, BlockTypeReader, ChunkBlockPos, ChunkLayoutKind, ChunkOffsets, ColumnExtras, HandleStamp, LoaderState,
    MapColorAdapter, SubChunkLayout, SurfaceAdapter, SurfaceReader, WorldBlockPos,
};

/// `LevelChunk* BlockSource::getChunk(int, int) const`: rcx the block source, the position in edx and r8d.
type GetChunkFn = unsafe extern "C" fn(*mut c_void, i32, i32) -> *mut c_void;
/// `Block const& LevelChunk::getBlock(ChunkBlockPos const&) const`: rcx the chunk, the position in rdx.
type GetBlockFn = unsafe extern "C" fn(*mut c_void, *const ChunkBlockPos) -> *mut c_void;
/// `Biome const& LevelChunk::getBiome(ChunkBlockPos const&) const`: rcx the chunk, the position in rdx.
type GetBiomeFn = unsafe extern "C" fn(*mut c_void, *const ChunkBlockPos) -> *mut c_void;
/// `bool SubChunk::isPlaceHolderSubChunk() const`: rcx the sub-chunk, the bool in al.
type PlaceholderFn = unsafe extern "C" fn(*mut c_void) -> u8;

// The chunks given in one game frame.
struct Issued {
    frame: u64,
    chunks: HashSet<usize>,
}

/// The game side of the `world.chunk_read` area: the chunk of a block source, how much of it the client has, and the
/// walk of its columns. It holds no check of its own for the caller: `WorldAdapter` has made the gate, area, thread
/// and block source checks before a call gets here.
///
/// A chunk handle is the chunk address with the low 16 bits of the game frame in its top bits (`HandleStamp`), and
/// it is accepted only for a chunk that was given in the frame of the call, so a handle kept past its frame is
/// refused and never read.
pub struct ChunkAdapter {
    layout: ChunkLayoutKind,
    offsets: ChunkOffsets,
    get_chunk_slot: usize,
    get_block: usize,
    is_placeholder: usize,
    types: BlockTypeReader,
    // `LevelChunk::getBiome`; `None` when the build has no entry for it.
    get_biome: Option<usize>,
    // The map colours and the biome ids; `None` when the build lacks a part of their entries.
    colors: Option<MapColorAdapter>,
    biomes: Option<BiomeReader>,
    // The surface scan; `None` when the build lacks a part of its entries.
    surfaces: Option<SurfaceAdapter>,
    issued: Mutex<Issued>,
}

// What the read of the columns of one chunk shares.
struct ColumnFrame<'a> {
    chunk: usize,
    // `mMin` of the chunk: the world block position of its lowest corner.
    min: [i32; 3],
    // The highest block y of the chunk, relative to its lowest.
    top: i32,
    epoch: u64,
    extras: ColumnExtras,
    // The colour calls and the main chunk source of the dimension of the chunk, when the colours are asked for.
    paint: Option<(&'a MapColorAdapter, usize)>,
}

impl ChunkAdapter {
    // `ChunkState::LOADED`, the value of the one-byte `mLoadState` of a fully loaded chunk.
    const LOADED: u8 = 11;
    // The most chunks one frame gives; a mod reads a few dozen.
    const MAX_ISSUED: usize = 4096;
    const COLUMNS: usize = 256;
    // `mDimension` of the chunk, on both layouts.
    const DIMENSION_AT: usize = 0x58;

    /// The adapter for the entries of the build; `None` when one is missing or has a value that no mirror has. A
    /// `LevelChunk.mSubChunks` value that no layout has is logged as `chunk layout unknown`.
    pub fn new(state: &LoaderState) -> Option<Self> {
        let value = |name: &str| state.resolve(name).ok();
        let function = |name: &str| value(name).and_then(|value| usize::try_from(value).ok()).filter(|at| *at != 0);
        let sub_chunks = value("LevelChunk.mSubChunks")?;
        let Some(layout) = ChunkLayoutKind::of(sub_chunks) else {
            curia::warn!("chunk layout unknown", { value: sub_chunks });
            return None;
        };
        if value("SubChunk.size")? != u64::try_from(SubChunkLayout::SIZE).ok()? {
            curia::warn!("chunk sub-chunk size unknown", { value: value("SubChunk.size").unwrap_or(0) });
            return None;
        }
        let types = BlockTypeReader::new(
            value("HashedString.layout")?,
            value("Block.mBlockType")?,
            value("BlockType.mNameInfo.mFullName")?,
            value("BlockType.mMaterialInline")?,
        )?;
        Some(Self {
            layout,
            offsets: layout.offsets(),
            get_chunk_slot: usize::try_from(value("BlockSource.getChunk")?).ok()?,
            get_block: function("LevelChunk.getBlock")?,
            is_placeholder: function("SubChunk.isPlaceHolderSubChunk")?,
            types,
            get_biome: function("LevelChunk.getBiome"),
            colors: MapColorAdapter::new(state),
            biomes: BiomeReader::new(state),
            surfaces: SurfaceAdapter::new(state),
            issued: Mutex::new(Issued { frame: 0, chunks: HashSet::new() }),
        })
    }

    /// Whether the build serves the area `area` (`world.map_color`, `world.biomes`, `world.scan_surfaces`) with this adapter.
    pub fn offers(&self, area: &str) -> bool {
        match area {
            "world.map_color" => self.colors.is_some() && self.get_biome.is_some(),
            "world.biomes" => self.biomes.is_some() && self.get_biome.is_some(),
            "world.scan_surfaces" => self.surfaces.is_some(),
            _ => true,
        }
    }

    /// The name of the biome `id` in the world `epoch`; `None` for an id the column reads did not meet in it.
    pub fn biome_name(&self, id: u16, epoch: u64) -> Option<String> {
        self.biomes.as_ref()?.name(id, epoch)
    }

    /// The layout of the game build.
    pub fn layout(&self) -> ChunkLayoutKind {
        self.layout
    }

    /// The block type reader, for the reads of a column that other adapters share.
    pub(crate) fn types(&self) -> &BlockTypeReader {
        &self.types
    }

    /// The name of the block type `id`; `None` for an id never given and a name that was not readable.
    pub fn type_name(&self, id: u32) -> Option<String> {
        self.types.type_name(id)
    }

    /// The handle of the chunk at chunk position (`x`, `z`) of the live block source `source`, for the game frame
    /// `frame`; 0 when the game has no chunk there.
    ///
    /// # Safety
    /// `source` is the live block source of the local player of the primary client, called on the game thread.
    pub unsafe fn chunk(&self, source: usize, x: i32, z: i32, frame: u64) -> Result<usize, AmmlStatus> {
        // SAFETY: as the function's contract.
        let chunk = unsafe { self.chunk_address(source, x, z) }?;
        if chunk == 0 {
            return Ok(0);
        }
        let handle = HandleStamp::seal(chunk, frame).ok_or(AmmlStatus::FAILED)?;
        let mut issued = self.lock();
        if issued.frame != frame {
            issued.frame = frame;
            issued.chunks.clear();
        }
        if issued.chunks.len() >= Self::MAX_ISSUED && !issued.chunks.contains(&chunk) {
            return Err(AmmlStatus::FAILED);
        }
        issued.chunks.insert(chunk);
        Ok(handle)
    }

    // The address of the chunk at chunk position (`x`, `z`) of the live block source `source`; 0 when the game has none.
    unsafe fn chunk_address(&self, source: usize, x: i32, z: i32) -> Result<usize, AmmlStatus> {
        // SAFETY: `source` is live and starts with its vtable; the entry is the slot of `getChunk` in it.
        let function = unsafe { Self::virtual_fn(source, self.get_chunk_slot) }.ok_or(AmmlStatus::NOT_AVAILABLE)?;
        // SAFETY: the slot holds `LevelChunk* BlockSource::getChunk(int, int) const`.
        let function = unsafe { std::mem::transmute::<usize, GetChunkFn>(function) };
        // SAFETY: called on the live block source, on the game thread.
        Ok(unsafe { function(std::ptr::with_exposed_provenance_mut(source), x, z) }.expose_provenance())
    }

    /// The address of the chunk `handle` names, when it was given in the game frame `frame`; `REFUSED` otherwise.
    pub fn open(&self, handle: usize, frame: u64) -> Result<usize, AmmlStatus> {
        let chunk = HandleStamp::open(handle, frame).ok_or(AmmlStatus::REFUSED)?;
        let issued = self.lock();
        if issued.frame == frame && issued.chunks.contains(&chunk) { Ok(chunk) } else { Err(AmmlStatus::REFUSED) }
    }

    /// Finds the surfaces of the column at block position (`x`, `z`) of the live block source `source` with a block y from
    /// `y_min` to `y_max` (both included), lowest first, and gives each to `emit`, at most `limit` of them. Gives the
    /// number found. `FAILED` when the chunk is not loaded or a block was not readable; the rule is `SurfaceRule`.
    ///
    /// # Safety
    /// `source` is the live block source of the local player of the primary client, called on the game thread.
    pub unsafe fn scan_surfaces(
        &self,
        source: usize,
        (x, z): (i32, i32),
        (y_min, y_max): (i32, i32),
        limit: usize,
        epoch: u64,
        emit: impl FnMut(SurfaceHit),
    ) -> Result<usize, AmmlStatus> {
        let surfaces = self.surfaces.as_ref().ok_or(AmmlStatus::NOT_AVAILABLE)?;
        // SAFETY: as the function's contract.
        let chunk = unsafe { self.chunk_address(source, x >> 4, z >> 4) }?;
        if chunk == 0 {
            return Err(AmmlStatus::FAILED);
        }
        // SAFETY: `chunk` is the live chunk the game just gave, on the game thread.
        if unsafe { self.load_state(chunk) } != AmmlChunkLoad::LOADED {
            return Err(AmmlStatus::FAILED);
        }
        // SAFETY: as above; `min` and `max` are the `BlockPos` members of the chunk, y second. `mMax.y` is the highest
        // block y.
        let (min_y, top_y) =
            unsafe { (Self::read::<i32>(chunk, self.offsets.min + 4), Self::read::<i32>(chunk, self.offsets.max + 4)) };
        let (Some(min_y), Some(top_y)) = (min_y, top_y) else { return Err(AmmlStatus::FAILED) };
        if top_y < min_y {
            return Err(AmmlStatus::FAILED);
        }
        let mut column = SurfaceReader { chunks: self, surfaces, chunk, source, x, z, min_y, epoch };
        SurfaceRule::new(min_y, top_y).scan(&mut column, y_min, y_max, limit, emit).ok_or(AmmlStatus::FAILED)
    }

    /// How much of the chunk the client has: not loaded, loaded with its top sub-chunk still the client-request
    /// placeholder, or loaded.
    ///
    /// # Safety
    /// `chunk` is a live chunk (`open` gave it in this frame), on the game thread.
    pub unsafe fn load_state(&self, chunk: usize) -> AmmlChunkLoad {
        // SAFETY: `chunk` is live and `load_state` is the offset of its `mLoadState` byte.
        let loaded = unsafe { Self::read::<u8>(chunk, self.offsets.load_state) } == Some(Self::LOADED);
        if !loaded {
            return AmmlChunkLoad::NOT_LOADED;
        }
        // SAFETY: as above, for the sub-chunk list.
        let top_ready = match unsafe { self.sub_chunks(chunk) } {
            Some((begin, count)) if count > 0 => {
                // SAFETY: `count - 1 < count`, so the sub-chunk is inside the vector of the live chunk.
                let top = unsafe { self.placeholder(begin, count - 1) };
                top == Some(false)
            }
            _ => false,
        };
        if top_ready { AmmlChunkLoad::LOADED } else { AmmlChunkLoad::TOP_PLACEHOLDER }
    }

    /// Reads the columns of `chunk` that `request` asks for into `out` (256 samples, column `z * 16 + x`). A column
    /// that was not asked for is not written. A chunk that is not loaded or whose top sub-chunk is a placeholder
    /// marks every asked column `SKIPPED`, as does a column whose surface sub-chunk is a placeholder.
    ///
    /// # Safety
    /// `chunk` is a live chunk (`open` gave it in this frame), on the game thread.
    pub unsafe fn read_columns(
        &self,
        chunk: usize,
        request: &AmmlColumnRequest,
        out: &mut [AmmlColumnSample],
        epoch: u64,
        extras: ColumnExtras,
    ) -> AmmlColumnReadStats {
        let mut stats = AmmlColumnReadStats::default();
        // SAFETY: as the function's contract.
        let load = unsafe { self.load_state(chunk) };
        // SAFETY: as the function's contract.
        let range = unsafe { self.height_range(chunk) }.filter(|_| load == AmmlChunkLoad::LOADED);
        // SAFETY: as the function's contract.
        let span = unsafe { self.sub_chunks(chunk) };
        let walk = range.map(|(min_y, max_y)| {
            (min_y, max_y, ColumnWalk::new(min_y, max_y, request.flags & AmmlColumnRequest::TRANSPARENT_WATER != 0))
        });
        // SAFETY: as the function's contract.
        let frame = walk.as_ref().map(|(min_y, max_y, _)| unsafe { self.frame(chunk, *min_y, *max_y, epoch, extras) });
        for index in 0..Self::COLUMNS {
            if (request.columns[index / 64] >> (index % 64)) & 1 == 0 {
                continue;
            }
            let Some(sample) = out.get_mut(index) else { continue };
            let read = match (&walk, &frame, span) {
                (Some((_, _, walk)), Some(frame), Some(span)) => {
                    // SAFETY: as the function's contract; `span` describes the sub-chunk vector of `chunk`.
                    unsafe { self.sample_column(frame, index, walk, request.camera_y, span) }
                }
                _ => None,
            };
            match read {
                Some(read) => {
                    *sample = read;
                    stats.read = stats.read.saturating_add(1);
                    if read.flags & AmmlColumnSample::RETRY != 0 {
                        stats.retry = stats.retry.saturating_add(1);
                    }
                }
                None => {
                    *sample = Self::skipped();
                    stats.skipped = stats.skipped.saturating_add(1);
                }
            }
        }
        stats
    }

    // The sample of a column that was not read.
    fn skipped() -> AmmlColumnSample {
        AmmlColumnSample {
            shown: 0,
            water: 0,
            shown_color: AmmlRgba { r: 0, g: 0, b: 0, a: 0 },
            water_color: AmmlRgba { r: 0, g: 0, b: 0, a: 0 },
            shown_y: 0,
            height: 0,
            solid: 0,
            biome: AmmlColumnSample::NO_BIOME,
            water_depth: 0,
            flags: AmmlColumnSample::SKIPPED,
            _reserved: [0; 2],
        }
    }

    // The shared state of the column reads of the live chunk whose block range is `min_y` to `max_end` (exclusive): its
    // corner, and the colour source when the colours are asked for and the dimension gives a main chunk source.
    unsafe fn frame(
        &self,
        chunk: usize,
        min_y: i32,
        max_end: i32,
        epoch: u64,
        extras: ColumnExtras,
    ) -> ColumnFrame<'_> {
        // SAFETY: `chunk` is live and `min` is the offset of its `mMin`, three ints.
        let (min_x, min_z) = unsafe {
            (
                Self::read::<i32>(chunk, self.offsets.min).unwrap_or(0),
                Self::read::<i32>(chunk, self.offsets.min + 8).unwrap_or(0),
            )
        };
        let paint = self.colors.as_ref().filter(|_| extras.colors && self.get_biome.is_some()).and_then(|colors| {
            // SAFETY: `chunk` is live and `DIMENSION_AT` is its `mDimension` pointer.
            let dimension = unsafe { Self::read::<usize>(chunk, Self::DIMENSION_AT) }?;
            // SAFETY: the dimension of a live chunk is live; called on the game thread.
            let source = unsafe { colors.main_source(dimension) }?;
            Some((colors, source))
        });
        ColumnFrame { chunk, min: [min_x, min_y, min_z], top: (max_end - 1 - min_y).max(0), epoch, extras, paint }
    }

    // Walks column `index`; `None` when its surface sub-chunk is still a placeholder or its heights cannot be read.
    unsafe fn sample_column(
        &self,
        frame: &ColumnFrame<'_>,
        index: usize,
        walk: &ColumnWalk,
        camera_y: i32,
        (begin, count): (usize, usize),
    ) -> Option<AmmlColumnSample> {
        let (chunk, min_y) = (frame.chunk, frame.min[1]);
        // SAFETY: `chunk` is live and `index < 256`, inside both heightmaps.
        let height = unsafe { Self::read::<i16>(chunk, self.offsets.heightmap + index * 2) }?;
        // SAFETY: as above.
        let render = unsafe { Self::read::<i16>(chunk, self.offsets.render_heightmap + index * 2) }?;
        let sub = (i32::from(height) >> 4).min(i32::try_from(count).ok()? - 1);
        // SAFETY: `sub < count` when it is not negative, so the sub-chunk is inside the vector.
        if sub >= 0 && unsafe { self.placeholder(begin, usize::try_from(sub).ok()?) } == Some(true) {
            return None;
        }
        let (x, z) = ((index % 16) as u8, (index / 16) as u8);
        let mut column = ChunkColumn { adapter: self, chunk, x, z, min_y, epoch: frame.epoch };
        let walked = walk.walk(&mut column, camera_y, i32::from(height) + min_y);
        let mut flags = 0;
        if walked.retry {
            flags |= AmmlColumnSample::RETRY;
        }
        if walked.void {
            flags |= AmmlColumnSample::VOID;
        }
        // The biome of the column surface: the block under the height.
        let surface = ChunkBlockPos { x, z, y: (i32::from(height) - 1).clamp(0, frame.top) as i16 };
        let biome = if frame.extras.biomes || frame.paint.is_some() {
            // SAFETY: `chunk` is live and the position is inside it.
            unsafe { self.biome_at(chunk, surface) }
        } else {
            0
        };
        let id = if frame.extras.biomes {
            self.biomes.as_ref().and_then(|reader| {
                // SAFETY: the game gives a live biome of the chunk, or null.
                unsafe { reader.id(biome, frame.epoch) }
            })
        } else {
            None
        };
        let (shown_color, water_color) = match frame.paint {
            // SAFETY: `chunk` and the colour source are live; called on the game thread.
            Some((colors, source)) => unsafe { self.paint(frame, colors, source, (x, z), &walked, biome) },
            None => (None, None),
        };
        // A shown block with no colour, and a void column that was asked for a colour and has none (no biome to give the
        // map grass colour).
        let no_color = if walked.shown != 0 {
            frame.paint.is_none() || shown_color.is_none()
        } else {
            walked.void && frame.extras.colors && shown_color.is_none()
        };
        if no_color {
            flags |= AmmlColumnSample::NO_COLOR;
        }
        let none = AmmlRgba { r: 0, g: 0, b: 0, a: 0 };
        Some(AmmlColumnSample {
            shown: walked.shown,
            water: walked.water,
            shown_color: shown_color.unwrap_or(none),
            water_color: water_color.unwrap_or(none),
            shown_y: Self::clamp_y(walked.shown_y),
            height: Self::clamp_y(i32::from(height) + min_y),
            solid: Self::clamp_y(i32::from(render) + min_y),
            biome: id.unwrap_or(AmmlColumnSample::NO_BIOME),
            water_depth: walked.water_depth,
            flags,
            _reserved: [0; 2],
        })
    }

    // The colours of a walked column: the shown block, whose colour is the biome water colour for a water block and the
    // map grass colour of the biome for a column with no block, and the colour of the water above it. A colour the game
    // does not give is `None`. `surface` is the biome of the column surface.
    unsafe fn paint(
        &self,
        frame: &ColumnFrame<'_>,
        colors: &MapColorAdapter,
        source: usize,
        (x, z): (u8, u8),
        walked: &WalkedColumn,
        surface: usize,
    ) -> (Option<AmmlRgba>, Option<AmmlRgba>) {
        let [min_x, min_y, min_z] = frame.min;
        let world = |y: i32| WorldBlockPos { x: min_x + i32::from(x), y, z: min_z + i32::from(z) };
        let local = |y: i32| ChunkBlockPos { x, z, y: (y - min_y).clamp(0, frame.top) as i16 };
        let shown = if walked.shown != 0 {
            // SAFETY: `chunk` is live; the walk read this block position, on the game thread.
            let block = unsafe { self.block_at(frame.chunk, local(walked.shown_y)) };
            // SAFETY: the game gives a live Block, or null.
            let shape = unsafe { self.types.walk_block(block, frame.epoch) };
            if shape.is_some_and(|shape| shape.is_water) {
                // SAFETY: the chunk is live; the game gives a live biome, or null.
                let biome = unsafe { self.biome_at(frame.chunk, local(walked.shown_y)) };
                // SAFETY: the biome is live.
                (biome != 0).then(|| unsafe { colors.biome_water(biome, &world(walked.shown_y)) })
            } else if block != 0 {
                // SAFETY: the block is live and the source is the main chunk source of the dimension of the chunk.
                unsafe { colors.block_color(block, source, &world(walked.shown_y)) }
            } else {
                None
            }
        } else if walked.void {
            // SAFETY: the chunk is live; the game gives a live biome, or null.
            let biome = unsafe { self.biome_at(frame.chunk, local(min_y)) };
            // SAFETY: the biome is live.
            (biome != 0).then(|| unsafe { colors.biome_grass(biome, &world(min_y)) })
        } else {
            None
        };
        let water = (walked.water != 0 && surface != 0).then(|| {
            let top = walked.shown_y.saturating_add(i32::from(walked.water_depth));
            // SAFETY: the surface biome is live.
            unsafe { colors.biome_water(surface, &world(top)) }
        });
        (shown, water)
    }

    // The biome at `pos` of the live chunk; 0 for none.
    unsafe fn biome_at(&self, chunk: usize, pos: ChunkBlockPos) -> usize {
        let Some(function) = self.get_biome else { return 0 };
        // SAFETY: the entry is `Biome const& LevelChunk::getBiome(ChunkBlockPos const&) const`.
        let function = unsafe { std::mem::transmute::<usize, GetBiomeFn>(function) };
        // SAFETY: called on the live chunk with a position that lives for the call, on the game thread.
        unsafe { function(std::ptr::with_exposed_provenance_mut(chunk), &raw const pos) }.expose_provenance()
    }

    fn clamp_y(y: i32) -> i16 {
        y.clamp(i32::from(i16::MIN), i32::from(i16::MAX)) as i16
    }

    // The lowest block y and the end (one past `mMax.y`) of the block range of the live chunk; `None` for a reversed
    // range.
    unsafe fn height_range(&self, chunk: usize) -> Option<(i32, i32)> {
        // SAFETY: `chunk` is live; `min` and `max` are its `BlockPos` members, y second.
        let (min_y, max_y) = unsafe {
            (Self::read::<i32>(chunk, self.offsets.min + 4)?, Self::read::<i32>(chunk, self.offsets.max + 4)?)
        };
        // `mMax.y` is the highest block y; the walk takes the end of the range.
        (max_y >= min_y).then(|| (min_y, max_y.saturating_add(1)))
    }

    // The begin address and the count of the sub-chunk vector of the live chunk; `None` for a vector with no storage
    // or pointers that are not consistent.
    unsafe fn sub_chunks(&self, chunk: usize) -> Option<(usize, usize)> {
        // SAFETY: `chunk` is live; `sub_chunks` is its vector, begin then end.
        let (begin, end) = unsafe {
            (
                Self::read::<usize>(chunk, self.offsets.sub_chunks)?,
                Self::read::<usize>(chunk, self.offsets.sub_chunks + size_of::<usize>())?,
            )
        };
        let bytes = end.checked_sub(begin)?;
        (begin != 0).then_some((begin, bytes / SubChunkLayout::SIZE))
    }

    // Whether sub-chunk `index` of the vector at `begin` is the client-request placeholder; `None` when the call is
    // not possible.
    unsafe fn placeholder(&self, begin: usize, index: usize) -> Option<bool> {
        let at = begin.checked_add(index.checked_mul(SubChunkLayout::SIZE)?)?;
        // SAFETY: the entry is `bool SubChunk::isPlaceHolderSubChunk() const`.
        let function = unsafe { std::mem::transmute::<usize, PlaceholderFn>(self.is_placeholder) };
        // SAFETY: `at` is a sub-chunk inside the vector of a live chunk; called on the game thread.
        Some(unsafe { function(std::ptr::with_exposed_provenance_mut(at)) } != 0)
    }

    // The block at `pos` of the live chunk; 0 for none.
    pub(crate) unsafe fn block_at(&self, chunk: usize, pos: ChunkBlockPos) -> usize {
        // SAFETY: the entry is `Block const& LevelChunk::getBlock(ChunkBlockPos const&) const`.
        let function = unsafe { std::mem::transmute::<usize, GetBlockFn>(self.get_block) };
        // SAFETY: called on the live chunk with a position that lives for the call, on the game thread.
        unsafe { function(std::ptr::with_exposed_provenance_mut(chunk), &raw const pos) }.expose_provenance()
    }

    // The function in `slot` of the vtable at the start of `object`.
    unsafe fn virtual_fn(object: usize, slot: usize) -> Option<usize> {
        // SAFETY: `object` is a live object whose first qword is its vtable pointer.
        let vtable = unsafe { Self::read::<usize>(object, 0) }?;
        if vtable == 0 {
            return None;
        }
        // SAFETY: `vtable` is the vtable of the object; the entry keeps `slot` inside it.
        let function = unsafe { Self::read::<usize>(vtable, slot.checked_mul(size_of::<usize>())?) }?;
        (function != 0).then_some(function)
    }

    // The `T` at `offset` in the live object at `base`.
    unsafe fn read<T: Copy>(base: usize, offset: usize) -> Option<T> {
        let address = base.checked_add(offset)?;
        // SAFETY: the caller guarantees `address` is inside a live game object.
        Some(unsafe { std::ptr::with_exposed_provenance::<T>(address).read_unaligned() })
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Issued> {
        self.issued.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

// The blocks of one column of a live chunk, for the walk.
struct ChunkColumn<'a> {
    adapter: &'a ChunkAdapter,
    chunk: usize,
    x: u8,
    z: u8,
    min_y: i32,
    epoch: u64,
}

impl ColumnBlocks for ChunkColumn<'_> {
    fn block(&mut self, y: i32) -> Option<WalkBlock> {
        let pos = ChunkBlockPos { x: self.x, z: self.z, y: i16::try_from(y - self.min_y).ok()? };
        // SAFETY: the column belongs to a live chunk, read on the game thread (`read_columns`' contract).
        let block = unsafe { self.adapter.block_at(self.chunk, pos) };
        // SAFETY: the game gives a live Block, or null.
        unsafe { self.adapter.types.walk_block(block, self.epoch) }
    }
}
