use std::ffi::c_void;
use std::sync::OnceLock;

use amml_abi::{
    AmmlAabb, AmmlActorBox, AmmlBlockHit, AmmlBlockPos, AmmlChunkLoad, AmmlColumnReadStats, AmmlColumnRequest,
    AmmlColumnSample, AmmlRayOptions, AmmlStatus, AmmlSurfaceSample, AmmlThread, AmmlVec3,
};
use amml_data::{AdapterCatalog, Availability, EntryId, NeedKind};

use crate::{
    ActorAdapter, ActorsInAdapter, BlockTypeReader, ChunkAdapter, ClientAdapter, ColumnExtras, HandleStamp,
    InventoryAdapter, LoaderState, RaycastAdapter, ThreadNote,
};

static ADAPTER: OnceLock<WorldAdapter> = OnceLock::new();

// The calls of the world areas that note the thread they run on, one note each.
#[derive(Clone, Copy)]
enum Hook {
    SkyDarken,
    Chunk,
    LoadState,
    ReadColumns,
    SourceCheck,
    Scan,
    Raycast,
    ActorsIn,
}

impl Hook {
    // The game entry the call stands for, as the `world thread` line names it.
    fn entry(self) -> &'static str {
        match self {
            Self::SkyDarken => "Dimension.mSkyDarken",
            Self::Chunk => "BlockSource.getChunk",
            Self::LoadState => "SubChunk.isPlaceHolderSubChunk",
            Self::ReadColumns => "LevelChunk.getBlock",
            Self::SourceCheck => "Dimension.getBlockSourceFromMainChunkSource",
            Self::Scan => "BlockSource.getBrightnessPair",
            Self::Raycast => "BlockSource.clip",
            Self::ActorsIn => "BlockSource.fetchEntities",
        }
    }
}

/// The `world.*` areas: reads of the world of the primary client's local player.
///
/// Every call needs the game gate open (`NOT_READY`), the area offered (`NOT_AVAILABLE`), the game thread and a
/// handle that is the current one of the local player of the primary client (`REFUSED`). A handle is valid in the
/// game frame of the call that gave it; the check on each call turns a handle kept past a dimension change, a
/// world leave or a client change into `REFUSED`, never a read.
///
/// The block source handle carries the world epoch (`HandleStamp`), so a handle from before a world leave is refused
/// also when the game has put a block source at the same address. A chunk handle carries the game frame and is
/// accepted only for a chunk given in the frame of the call.
///
/// `world.raycast` and `world.actors_in` are offered only when their game calls resolved (`RaycastAdapter`,
/// `ActorsInAdapter`).
pub struct WorldAdapter {
    state: &'static LoaderState,
    // Offset of `mSkyDarken` in Dimension.
    sky_darken_at: Option<usize>,
    // The chunk reads; `None` when the build has no chunk layout the loader knows.
    chunk: Option<ChunkAdapter>,
    // The ray walk and the actor fetch; `None` when their entries did not resolve.
    raycast: Option<RaycastAdapter>,
    actors: Option<ActorsInAdapter>,
    // The thread the calls of each hook ran on, logged once and again on a change.
    threads: [ThreadNote; 8],
}

impl WorldAdapter {
    pub const SKY_DARKEN: &'static str = "world.sky_darken";
    pub const CHUNK_READ: &'static str = "world.chunk_read";
    pub const MAP_COLOR: &'static str = "world.map_color";
    pub const BIOMES: &'static str = "world.biomes";
    pub const SCAN_SURFACES: &'static str = "world.scan_surfaces";
    pub const RAYCAST: &'static str = "world.raycast";
    pub const ACTORS_IN: &'static str = "world.actors_in";
    // The only version of `world.raycast` and of `world.actors_in`.
    const RAY_VERSION: u32 = 1;
    // The only version of `world.scan_surfaces`.
    const SCAN_SURFACES_VERSION: u32 = 1;
    // The only version of `world.map_color` and of `world.biomes`.
    const COLUMN_EXTRA_VERSION: u32 = 1;
    // The only version of `world.chunk_read`.
    const CHUNK_READ_VERSION: u32 = 1;
    // The columns of a chunk, and the samples `read_columns` takes.
    pub const COLUMNS: u32 = 256;

    /// Offers every `world.*` area whose catalog shape resolved and logs one `area adapter` or `area not available`
    /// line per area. Returns the rows. Works once per process: a second call offers nothing and gives the catalog
    /// rows. `world.chunk_read` is offered only when the build has a `LevelChunk` layout the loader knows.
    pub fn install(state: &'static LoaderState) -> Vec<Availability> {
        let resolved = |name: &str| state.table().get(&EntryId(name.to_string())).is_some();
        let rows: Vec<Availability> = AdapterCatalog::availability(&resolved)
            .into_iter()
            .filter(|row| row.kind == NeedKind::Area && row.id.starts_with("world."))
            .collect();
        let slot = |name: &str| state.resolve(name).ok().and_then(|value| usize::try_from(value).ok());
        let chunk_resolved = rows.iter().any(|row| row.id == Self::CHUNK_READ && row.shape.is_some());
        let adapter = Self {
            state,
            sky_darken_at: slot("Dimension.mSkyDarken"),
            chunk: if chunk_resolved { ChunkAdapter::new(state) } else { None },
            raycast: RaycastAdapter::new(state),
            actors: ActorsInAdapter::new(state),
            threads: [const { ThreadNote::new() }; 8],
        };
        let ready: Vec<bool> = rows.iter().map(|row| adapter.ready(row.id)).collect();
        if ADAPTER.set(adapter).is_err() {
            curia::warn!("the world adapter is already installed");
            return rows;
        }
        for (row, ready) in rows.iter().zip(ready) {
            match row.shape {
                Some(shape) if ready => {
                    state.areas().offer(row.id, row.version);
                    curia::info!("area adapter", { area: row.id, shape: shape });
                }
                Some(_) => {
                    let missing = if row.id == Self::RAYCAST || row.id == Self::ACTORS_IN {
                        "ray adapter"
                    } else {
                        "chunk adapter"
                    };
                    curia::info!("area not available", { area: row.id, missing: missing });
                }
                None => curia::info!("area not available", { area: row.id, missing: row.missing.join(", ") }),
            }
        }
        rows
    }

    // Whether the area `area` has what it needs: every chunk area needs the chunk reads, and the column extras need
    // their own calls; the ray and the actor areas need their game calls.
    fn ready(&self, area: &str) -> bool {
        if area == Self::RAYCAST {
            return self.raycast.is_some();
        }
        if area == Self::ACTORS_IN {
            return self.actors.is_some();
        }
        let chunk_area =
            area == Self::CHUNK_READ || area == Self::MAP_COLOR || area == Self::BIOMES || area == Self::SCAN_SURFACES;
        !chunk_area || self.chunk.as_ref().is_some_and(|chunk| chunk.offers(area))
    }

    /// The installed adapter; `None` before `install`.
    pub fn get() -> Option<&'static WorldAdapter> {
        ADAPTER.get()
    }

    /// The sky darkness of `dimension`, which must be the dimension of the local player of the primary client now
    /// (`REFUSED` otherwise): the byte of `mSkyDarken`.
    pub fn sky_darken(&self, dimension: *const c_void) -> Result<u8, AmmlStatus> {
        self.enter(Self::SKY_DARKEN, 1, Hook::SkyDarken)?;
        let at = self.sky_darken_at.ok_or(AmmlStatus::NOT_AVAILABLE)?;
        let dimension = dimension.expose_provenance();
        let actor = ActorAdapter::get().ok_or(AmmlStatus::NOT_AVAILABLE)?;
        if dimension == 0 || actor.primary_dimension()? != dimension {
            return Err(AmmlStatus::REFUSED);
        }
        let address = dimension.checked_add(at).ok_or(AmmlStatus::FAILED)?;
        // SAFETY: `dimension` is the live dimension of the local player (checked above) and `at` the derived
        // offset of its one-byte `mSkyDarken` member.
        Ok(unsafe { std::ptr::with_exposed_provenance::<u8>(address).read() })
    }

    /// The address of `block_source` when it is the current block source of the local player of the primary client
    /// (`Dimension::getBlockSourceFromMainChunkSource` of its dimension) and was given in the current world, for a
    /// world call of `area` at `version`. `REFUSED` for a null handle, a handle of another dimension, another world
    /// or a client that is gone, and when there is no local player.
    pub fn block_source(&self, area: &str, version: u32, block_source: *const c_void) -> Result<usize, AmmlStatus> {
        self.checked_source(area, version, block_source, Hook::SourceCheck)
    }

    /// The chunk at chunk position (`x`, `z`) of `source`, a block source handle of the current world: a handle for
    /// the chunk calls of this game frame, or 0 when the game has no chunk there. Area `world.chunk_read`.
    pub fn chunk(&self, source: *const c_void, x: i32, z: i32) -> Result<usize, AmmlStatus> {
        let source = self.checked_source(Self::CHUNK_READ, Self::CHUNK_READ_VERSION, source, Hook::Chunk)?;
        let chunk = self.chunk.as_ref().ok_or(AmmlStatus::NOT_AVAILABLE)?;
        // SAFETY: `source` is the live block source of the local player of the primary client (checked above), and
        // the call runs on the game thread.
        unsafe { chunk.chunk(source, x, z, InventoryAdapter::frame()) }
    }

    /// How much of `chunk`, a chunk handle of this game frame, the client has. Area `world.chunk_read`.
    pub fn chunk_load_state(&self, chunk: *const c_void) -> Result<AmmlChunkLoad, AmmlStatus> {
        let (adapter, chunk) = self.open_chunk(chunk, Hook::LoadState)?;
        // SAFETY: `chunk` is a live chunk given in this frame, on the game thread.
        Ok(unsafe { adapter.load_state(chunk) })
    }

    /// Reads the columns of `chunk`, a chunk handle of this game frame, that `request` asks for into `out`. `len`
    /// must be 256. Area `world.chunk_read`.
    pub fn read_columns(
        &self,
        chunk: *const c_void,
        request: &AmmlColumnRequest,
        out: &mut [AmmlColumnSample],
    ) -> Result<AmmlColumnReadStats, AmmlStatus> {
        if out.len() != Self::COLUMNS as usize {
            return Err(AmmlStatus::INVALID_ARGUMENT);
        }
        let (adapter, chunk) = self.open_chunk(chunk, Hook::ReadColumns)?;
        let areas = self.state.areas();
        let extras = ColumnExtras {
            colors: request.flags & AmmlColumnRequest::VANILLA_COLORS != 0
                && areas.is_available(Self::MAP_COLOR, Self::COLUMN_EXTRA_VERSION),
            biomes: areas.is_available(Self::BIOMES, Self::COLUMN_EXTRA_VERSION),
        };
        // SAFETY: `chunk` is a live chunk given in this frame, on the game thread.
        Ok(unsafe { adapter.read_columns(chunk, request, out, self.state.events().gate().epoch().0, extras) })
    }

    /// Finds the surfaces of the column at block position (`x`, `z`) of `source`, a block source handle of the current
    /// world, with a block y from `y_min` to `y_max` (both included), lowest first, into `out`. Gives the number written.
    /// `FAILED` when the chunk is not loaded or a block was not readable. Area `world.scan_surfaces`.
    pub fn scan_surfaces(
        &self,
        source: *const c_void,
        column: (i32, i32),
        range: (i32, i32),
        out: &mut [AmmlSurfaceSample],
    ) -> Result<u32, AmmlStatus> {
        let source = self.checked_source(Self::SCAN_SURFACES, Self::SCAN_SURFACES_VERSION, source, Hook::Scan)?;
        let chunk = self.chunk.as_ref().ok_or(AmmlStatus::NOT_AVAILABLE)?;
        let (x, z) = column;
        let mut written = 0usize;
        let epoch = self.state.events().gate().epoch().0;
        let limit = out.len();
        // SAFETY: `source` is the live block source of the local player of the primary client (checked above), and the
        // call runs on the game thread.
        let found = unsafe {
            chunk.scan_surfaces(source, column, range, limit, epoch, |hit| {
                if let Some(slot) = out.get_mut(written) {
                    *slot = AmmlSurfaceSample {
                        pos: AmmlBlockPos { x, y: hit.y, z },
                        block_light: hit.block_light,
                        sky_light: hit.sky_light,
                        _reserved: [0; 2],
                    };
                }
                written += 1;
            })
        }?;
        u32::try_from(found).map_err(|_| AmmlStatus::FAILED)
    }

    /// The first block the ray from `from` to `to` meets in `source`, a block source handle of the current world; `hit`
    /// 0 for none. With `every_block` 0 only the blocks whose material is solid are candidates, which needs the block
    /// types of `world.chunk_read` (`NOT_AVAILABLE` without it). The points must be finite. Area `world.raycast`.
    pub fn raycast(
        &self,
        source: *const c_void,
        from: AmmlVec3,
        to: AmmlVec3,
        options: AmmlRayOptions,
    ) -> Result<AmmlBlockHit, AmmlStatus> {
        let source = self.checked_source(Self::RAYCAST, Self::RAY_VERSION, source, Hook::Raycast)?;
        let raycast = self.raycast.as_ref().ok_or(AmmlStatus::NOT_AVAILABLE)?;
        if options.every_block != 0 {
            // SAFETY: `source` is the live block source of the local player of the primary client (checked above), and
            // the call runs on the game thread.
            return unsafe { raycast.clip(source, from, to, options, |_| true) };
        }
        if !self.state.areas().is_available(Self::CHUNK_READ, Self::CHUNK_READ_VERSION) {
            return Err(AmmlStatus::NOT_AVAILABLE);
        }
        let types = self.block_types().ok_or(AmmlStatus::NOT_AVAILABLE)?;
        let epoch = self.state.events().gate().epoch().0;
        let solid = |block: usize| {
            // SAFETY: the game passes each live `Block` the walk reaches, on the game thread, during the clip call.
            let shape = unsafe { types.walk_block(block, epoch) };
            shape.is_some_and(|shape| shape.is_solid && !shape.is_air)
        };
        // SAFETY: as above.
        unsafe { raycast.clip(source, from, to, options, solid) }
    }

    /// Writes the boxes of the actors the game keeps in `source`, a block source handle of the current world, whose
    /// boxes meet `aabb` to `out`, and gives the number written. The local player of the primary client is marked. The
    /// box must be finite. Area `world.actors_in`.
    pub fn actors_in(
        &self,
        source: *const c_void,
        aabb: &AmmlAabb,
        out: &mut [AmmlActorBox],
    ) -> Result<u32, AmmlStatus> {
        let source = self.checked_source(Self::ACTORS_IN, Self::RAY_VERSION, source, Hook::ActorsIn)?;
        let actors = self.actors.as_ref().ok_or(AmmlStatus::NOT_AVAILABLE)?;
        let local_player = ClientAdapter::get().and_then(ClientAdapter::primary_local_player);
        // SAFETY: `source` is the live block source of the local player of the primary client (checked above), and the
        // call runs on the game thread.
        unsafe { actors.actors_in(source, aabb, local_player, out) }
    }

    /// The reader of block types the chunk reads use; `None` when the build has no chunk layout the loader knows.
    pub fn block_types(&self) -> Option<&BlockTypeReader> {
        self.chunk.as_ref().map(ChunkAdapter::types)
    }

    /// The name of the block type `id`. Any thread; needs the area only. `INVALID_ARGUMENT` for an id never given or a
    /// name that was not readable.
    pub fn block_type_name(&self, id: u32) -> Result<String, AmmlStatus> {
        if !self.state.areas().is_available(Self::CHUNK_READ, Self::CHUNK_READ_VERSION) {
            return Err(AmmlStatus::NOT_AVAILABLE);
        }
        let chunk = self.chunk.as_ref().ok_or(AmmlStatus::NOT_AVAILABLE)?;
        chunk.type_name(id).ok_or(AmmlStatus::INVALID_ARGUMENT)
    }

    /// The name of the biome `id` as the column reads of the current world met it. Any thread; needs the area only.
    /// `INVALID_ARGUMENT` for an id not met in the current world or a name that was not readable.
    pub fn biome_name(&self, id: u16) -> Result<String, AmmlStatus> {
        if !self.state.areas().is_available(Self::BIOMES, Self::COLUMN_EXTRA_VERSION) {
            return Err(AmmlStatus::NOT_AVAILABLE);
        }
        let chunk = self.chunk.as_ref().ok_or(AmmlStatus::NOT_AVAILABLE)?;
        chunk.biome_name(id, self.state.events().gate().epoch().0).ok_or(AmmlStatus::INVALID_ARGUMENT)
    }

    // The chunk adapter and the address of the chunk `handle` names, for a call of `world.chunk_read`.
    fn open_chunk(&self, handle: *const c_void, hook: Hook) -> Result<(&ChunkAdapter, usize), AmmlStatus> {
        self.enter(Self::CHUNK_READ, Self::CHUNK_READ_VERSION, hook)?;
        let adapter = self.chunk.as_ref().ok_or(AmmlStatus::NOT_AVAILABLE)?;
        let chunk = adapter.open(handle.expose_provenance(), InventoryAdapter::frame())?;
        Ok((adapter, chunk))
    }

    // The block source address behind `handle` for a call of `area` at `version`.
    fn checked_source(&self, area: &str, version: u32, handle: *const c_void, hook: Hook) -> Result<usize, AmmlStatus> {
        self.enter(area, version, hook)?;
        let epoch = self.state.events().gate().epoch().0;
        let block_source = HandleStamp::open(handle.expose_provenance(), epoch).ok_or(AmmlStatus::REFUSED)?;
        let actor = ActorAdapter::get().ok_or(AmmlStatus::NOT_AVAILABLE)?;
        if actor.primary_block_source()? != block_source {
            return Err(AmmlStatus::REFUSED);
        }
        Ok(block_source)
    }

    // The gate, the area and the thread of a world call: `NOT_READY`, `NOT_AVAILABLE`, `REFUSED`. The thread of the
    // call is logged the first time and when it changes.
    fn enter(&self, area: &str, version: u32, hook: Hook) -> Result<(), AmmlStatus> {
        let gate = self.state.events().gate();
        gate.require_open()?;
        if !self.state.areas().is_available(area, version) {
            return Err(AmmlStatus::NOT_AVAILABLE);
        }
        let on_game_thread = gate.on_game_thread();
        let thread = if on_game_thread { AmmlThread::GAME } else { AmmlThread::RENDER };
        if self.threads[hook as usize].changed(thread) {
            curia::info!("world thread", { hook: hook.entry(), game: on_game_thread });
        }
        if !on_game_thread {
            return Err(AmmlStatus::REFUSED);
        }
        Ok(())
    }
}
