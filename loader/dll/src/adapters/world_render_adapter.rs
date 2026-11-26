use std::cell::Cell;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use amml_abi::{
    AmmlBlockHandle, AmmlColorQuad, AmmlEvent, AmmlMeshBuilt, AmmlMeshGroup, AmmlMeshId, AmmlMeshStyle,
    AmmlMesherOptions, AmmlPhase, AmmlStatus, AmmlThread, AmmlTransform, WorldRendered,
};
use amml_data::{AdapterCatalog, Availability, EntryId, NeedKind};
use amml_msvc::MsvcHeap;
use amml_runtime::{
    CacheWindow, MaterialChoice, MeshBook, MeshKey, MeshKind, MeshRefusal, RenderScope, WorldPlacement, WorldTransform,
};

use crate::{
    ActorAdapter, BlockHandles, BlockMesher, BlockReadAdapter, ClientAdapter, EventHub, LoaderState, MeshCalls,
    MeshParts, RenderCall, RenderLevelDetours, SessionAdapter, TextureAdapter, ThreadNote, ViewCameraReader,
};

static ADAPTER: OnceLock<WorldRenderAdapter> = OnceLock::new();

thread_local! {
    // The `renderLevel` call whose `WorldRendered` dispatch runs on this thread; all zero outside.
    static CALL: Cell<RenderCall> = const { Cell::new(RenderCall { renderer: 0, screen: 0, frame_object: 0 }) };
}

/// The world render: raises `WorldRendered` after the game's level pass, answers the canvas calls of the dispatch
/// and owns the `MeshBook`, the meshes the loader built and the world material patch.
///
/// The event is raised from the shared `renderLevel` detour (`RenderLevelDetours`) only while a world is joined and
/// a mod listens to it. A canvas call names the `frame_id` of the dispatch (`RenderScope`); any other frame, and a
/// call from another thread, is `REFUSED`.
///
/// A mod's meshes are freed when the world is left, the surface is lost or the render device reset, the mod is
/// disabled, and when the graphics mode switches (the `vibrant` value, read at a `ClientTick` while a `WorldRendered`
/// listener exists or a mesh is live, changes). After any of these the id is `STALE` at once. The game object of a
/// freed mesh is destroyed on the `renderLevel` thread at the next call, with one exception: at a render device reset
/// the queued meshes are dropped without the game destructor (their memory is kept), because their game buffers
/// belong to the device that ends.
///
/// It also answers the block mesher (`render.block_mesher`, offered only with the canvas): `BlockMesher` builds a grid of
/// the mod into a block mesh of this book, inside the dispatch, on the game thread.
pub struct WorldRenderAdapter {
    state: &'static LoaderState,
    meshes: MeshBook,
    camera: Option<ViewCameraReader>,
    mesh: Option<MeshParts>,
    mesher: Option<BlockMesher>,
    frames: AtomicU64,
    vibrant: AtomicBool,
    // Whether the last `ClientTick` read the graphics mode.
    reading: AtomicBool,
    active: bool,
    thread: ThreadNote,
    ghost_thread: ThreadNote,
    mesher_thread: ThreadNote,
}

impl WorldRenderAdapter {
    pub const WORLD_CANVAS: &'static str = "render.world_canvas";
    pub const BLOCK_MESHER: &'static str = "render.block_mesher";
    /// The most quads one mesh holds: two windings of 4 vertices each stay under the 16-bit index limit.
    pub const MAX_QUADS: u32 = 8191;
    // `camera_position` came with version 1, `build` and `draw` with version 2, `clip_matrix` and the draw of a block
    // mesh with version 3.
    const CAMERA_VERSION: u32 = 1;
    const MESH_VERSION: u32 = 2;
    const GHOST_VERSION: u32 = 3;
    const MESHER_VERSION: u32 = 1;
    pub const WATER_TINT: &'static str = "render.water_tint";
    const WATER_VERSION: u32 = 1;

    /// Hooks `LevelRenderer::renderLevel` (the one shared detour) when a row of this adapter resolved, and offers
    /// `render.world_rendered` and `render.world_canvas`. The canvas is offered only when every mirror of the mesh
    /// path fits its entry (else `world canvas layout mismatch` names the entry), and neither row without the hook.
    /// Logs one `event offered` / `area adapter` or `... not available` line per row. Works once per process.
    pub fn install(state: &'static LoaderState) -> Vec<Availability> {
        let resolved = |name: &str| state.table().get(&EntryId(name.to_string())).is_some();
        let mut rows: Vec<Availability> =
            AdapterCatalog::availability(&resolved).into_iter().filter(Self::is_own_row).collect();
        let canvas_resolved =
            rows.iter().any(|row| row.kind == NeedKind::Area && row.id == Self::WORLD_CANVAS && row.shape.is_some());
        let calls = if canvas_resolved {
            match MeshCalls::new(state) {
                Ok(calls) => Some(calls),
                Err(error) => {
                    curia::warn!("world canvas layout mismatch", { entry: error.entry(), detail: error.to_string() });
                    for row in rows.iter_mut().filter(|row| row.kind == NeedKind::Area) {
                        row.shape = None;
                        row.missing = vec![error.entry()];
                    }
                    None
                }
            }
        } else {
            None
        };
        let mesher = Self::mesher_for(state, &mut rows, calls.is_some());
        Self::water_for(&mut rows, mesher.as_ref());
        let hooked = rows.iter().any(|row| row.shape.is_some()) && RenderLevelDetours::install(state);
        let adapter = Self {
            state,
            meshes: MeshBook::new(),
            camera: ViewCameraReader::new(state),
            mesh: calls.map(MeshParts::new),
            mesher,
            frames: AtomicU64::new(0),
            vibrant: AtomicBool::new(false),
            reading: AtomicBool::new(false),
            active: hooked,
            thread: ThreadNote::new(),
            ghost_thread: ThreadNote::new(),
            mesher_thread: ThreadNote::new(),
        };
        if ADAPTER.set(adapter).is_err() {
            curia::warn!("the world render adapter is already installed");
            return rows;
        }
        for row in &mut rows {
            if row.shape.is_some() && !hooked {
                row.shape = None;
                row.missing = vec![RenderLevelDetours::RENDER_LEVEL];
            }
            match (row.kind, row.shape) {
                (NeedKind::Event, Some(shape)) => {
                    state.events().offer(row.id, row.version);
                    curia::info!("event offered", { event: row.id, shape: shape });
                }
                (NeedKind::Area, Some(shape)) => {
                    state.areas().offer(row.id, row.version);
                    curia::info!("area adapter", { area: row.id, shape: shape });
                }
                (NeedKind::Event, None) => {
                    curia::info!("event not available", { event: row.id, missing: row.missing.join(", ") });
                }
                (NeedKind::Area, None) => {
                    curia::info!("area not available", { area: row.id, missing: row.missing.join(", ") });
                }
            }
        }
        rows
    }

    /// The installed adapter; `None` before `install`.
    pub fn get() -> Option<&'static WorldRenderAdapter> {
        ADAPTER.get()
    }

    /// Runs `body` on the installed adapter; `fallback` before `install`.
    pub(crate) fn with<R>(fallback: R, body: impl FnOnce(&WorldRenderAdapter) -> R) -> R {
        match ADAPTER.get() {
            Some(adapter) => body(adapter),
            None => fallback,
        }
    }

    fn is_own_row(row: &Availability) -> bool {
        match row.kind {
            NeedKind::Event => row.id == WorldRendered::ID,
            NeedKind::Area => {
                row.id == Self::WORLD_CANVAS || row.id == Self::BLOCK_MESHER || row.id == Self::WATER_TINT
            }
        }
    }

    // The block mesher when its row resolved, the mesh path was made (`canvas`), the actor and block read adapters are
    // installed and the `BlockTessellator` mirror fits its entry; else its row loses its shape and names what is missing.
    fn mesher_for(state: &'static LoaderState, rows: &mut [Availability], canvas: bool) -> Option<BlockMesher> {
        let row = rows.iter_mut().find(|row| row.kind == NeedKind::Area && row.id == Self::BLOCK_MESHER)?;
        row.shape?;
        let missing = if !canvas {
            Some("world canvas mesh path")
        } else if ActorAdapter::get().is_none() || BlockReadAdapter::get().is_none() {
            Some("actor adapter, block read adapter")
        } else {
            None
        };
        let made = match missing {
            Some(reason) => Err(reason),
            None => BlockMesher::new(state),
        };
        match made {
            Ok(mesher) => Some(mesher),
            Err(entry) => {
                row.shape = None;
                row.missing = vec![entry];
                None
            }
        }
    }

    // The water tint row keeps its shape only with the block mesher, and the chunk reads that give a biome and the water
    // colour call; else it loses it and names what is missing.
    fn water_for(rows: &mut [Availability], mesher: Option<&BlockMesher>) {
        let Some(row) = rows.iter_mut().find(|row| row.kind == NeedKind::Area && row.id == Self::WATER_TINT) else {
            return;
        };
        if row.shape.is_none() {
            return;
        }
        let missing = match mesher {
            None => Some("render.block_mesher"),
            Some(mesher) if !mesher.offers_water() => Some("chunk adapter, biome reads"),
            Some(_) => None,
        };
        if let Some(reason) = missing {
            row.shape = None;
            row.missing = vec![reason];
        }
    }

    fn hub(&self) -> &EventHub {
        self.state.events()
    }

    fn check_area(&self, version: u32) -> Result<(), AmmlStatus> {
        if self.state.areas().is_available(Self::WORLD_CANVAS, version) {
            Ok(())
        } else {
            Err(AmmlStatus::NOT_AVAILABLE)
        }
    }

    // Tells the render detour what waits for its thread: a game mesh to destroy, a material to restore.
    fn wake(&self) {
        if self.meshes.has_due() || self.mesher.as_ref().is_some_and(BlockMesher::has_due) {
            RenderLevelDetours::set(RenderLevelDetours::MESH_DUE, true);
        }
        if let Some(parts) = &self.mesh {
            RenderLevelDetours::set(RenderLevelDetours::MATERIAL, parts.material.is_applied());
        }
    }

    fn request_restore(&self) {
        if let Some(parts) = &self.mesh {
            parts.material.request_restore();
        }
    }

    /// Whether the graphics mode is read at this `ClientTick`: the hook is in and a mod listens to `WorldRendered` or
    /// holds a mesh. When neither holds, the mode seen so far is forgotten, so the next first read records it without
    /// taking it as a switch.
    pub fn reads_graphics_mode(&self) -> bool {
        let wanted =
            self.active && (RenderLevelDetours::is_set(RenderLevelDetours::LISTENER) || !self.meshes.is_empty());
        if self.reading.swap(wanted, Ordering::AcqRel) && !wanted {
            self.meshes.forget_vibrant();
        }
        wanted
    }

    /// The graphics mode the game has now, read at a `ClientTick`. A change of the mode frees every mesh and asks for
    /// the material restore.
    pub fn set_vibrant(&self, vibrant: bool) {
        self.vibrant.store(vibrant, Ordering::Release);
        if self.meshes.note_vibrant(vibrant) {
            curia::info!("meshes freed", { reason: "graphics_mode", vibrant: vibrant });
            self.request_restore();
            self.wake();
        }
    }

    /// Before `WorldLeft`: every mesh is freed and its id is stale; the material restore is asked for.
    pub fn world_left(&self) {
        let count = self.meshes.world_left();
        if count > 0 {
            curia::info!("meshes freed", { reason: "world_left", count: count });
        }
        if let Some(mesher) = &self.mesher {
            mesher.world_left(self.state.events().gate().on_game_thread());
        }
        self.request_restore();
        self.wake();
    }

    /// The surface is lost or the render device was reset: every mesh is freed and its id is stale. The meshes that
    /// wait for the render thread are dropped here without the game destructor: their game buffers belong to the
    /// device that ends, and this call does not run on the `renderLevel` thread. Their memory is kept for the process.
    pub(crate) fn device_reset(&self) {
        let count = self.meshes.device_reset();
        if count > 0 {
            curia::info!("meshes freed", { reason: "device_reset", count: count });
        }
        if let Some(mesher) = &self.mesher {
            mesher.device_reset();
        }
        let dropped = self.meshes.take_due().len();
        if dropped > 0 {
            curia::info!("meshes dropped", { reason: "device_reset", count: dropped });
        }
        RenderLevelDetours::set(RenderLevelDetours::MESH_DUE, false);
        self.request_restore();
        self.wake();
    }

    /// The mod at `owner` is disabled: its meshes are freed; when it was the last mod that drew with the patched
    /// material, the restore is asked for.
    pub(crate) fn disable_owner(&self, owner: usize) {
        let count = self.meshes.disable_owner(owner);
        if count > 0 {
            curia::info!("meshes freed", { reason: "disable", count: count });
        }
        if let Some(mesher) = &self.mesher {
            mesher.disable_owner(owner);
        }
        if let Some(parts) = &self.mesh {
            parts.material.disable_owner(owner);
        }
        self.wake();
    }

    /// Before the game's `renderLevel`, on its thread: the game meshes that wait are destroyed with the
    /// game destructor; the book gives each handle once.
    pub(crate) fn destroy_due(&self) {
        if let Some(mesher) = &self.mesher {
            mesher.destroy_due();
        }
        let due = self.meshes.take_due();
        if due.is_empty() {
            return;
        }
        match &self.mesh {
            Some(parts) => {
                for free in &due {
                    // SAFETY: each handle is a mesh `MeshBuilder::build` made, given once by the book; this is the
                    // `renderLevel` thread, and a device reset would have taken the queue before.
                    unsafe { parts.builder.destroy(free.handle) };
                }
            }
            None => curia::warn!("meshes due without a mesh path", { count: due.len() }),
        }
        curia::debug!("meshes destroyed", { count: due.len() });
    }

    /// Before the game's `renderLevel` of `renderer`, on its thread, while the material is patched: writes the restore
    /// that is due (also at shutdown) into the material the running renderer has.
    pub(crate) fn before_level(&self, renderer: usize) {
        let Some(parts) = &self.mesh else { return };
        if self.state.shutdown().is_closing() {
            parts.material.request_restore();
        }
        // SAFETY: `renderer` is the level renderer of the running `renderLevel` call, on this thread.
        unsafe { parts.material.restore_due_now(renderer) };
        RenderLevelDetours::set(RenderLevelDetours::MATERIAL, parts.material.is_applied());
    }

    /// After the game's `renderLevel` of `call`: raises `WorldRendered` on this thread while a world is joined. The
    /// `frame_id` is new for each dispatch; no listener changes it, `vibrant` or the reserved bytes.
    pub(crate) fn rendered(&self, call: RenderCall) {
        let hub = self.hub();
        let gate = hub.gate();
        if !gate.is_open() || gate.epoch().0 == 0 {
            return;
        }
        let game = gate.on_game_thread();
        if self.thread.changed(if game { AmmlThread::GAME } else { AmmlThread::RENDER }) {
            curia::info!("world thread", { hook: RenderLevelDetours::RENDER_LEVEL, game: game });
        }
        let frame_id = self.frames.fetch_add(1, Ordering::Relaxed) + 1;
        let vibrant = u8::from(self.vibrant.load(Ordering::Acquire));
        let mut event = WorldRendered {
            header: EventHub::header::<WorldRendered>(AmmlPhase::AFTER),
            frame_id,
            vibrant,
            _reserved: [0; 7],
        };
        let mut context = hub.context_on(if game { AmmlThread::GAME } else { AmmlThread::RENDER });
        if game && let Some(client) = SessionAdapter::primary_client() {
            context.client = std::ptr::with_exposed_provenance(client);
        }
        let keep = |event: &mut WorldRendered| {
            event.frame_id = frame_id;
            event.vibrant = vibrant;
            event._reserved = [0; 7];
        };
        let _frame = RenderScope::enter(frame_id);
        let previous = CALL.with(|cell| cell.replace(call));
        hub.raise_keeping(&mut event, &context, &keep);
        CALL.with(|cell| cell.set(previous));
    }

    /// The camera position of the world view of `frame`. `NOT_AVAILABLE` without the area, `REFUSED` outside the
    /// dispatch of `frame` on this thread, `FAILED` when no view of the frame belongs to the running renderer.
    pub fn canvas_camera_position(&self, frame: u64) -> Result<[f32; 3], AmmlStatus> {
        self.check_area(Self::CAMERA_VERSION)?;
        if !RenderScope::allows(frame) {
            return Err(AmmlStatus::REFUSED);
        }
        self.frame_camera()
    }

    fn frame_camera(&self) -> Result<[f32; 3], AmmlStatus> {
        let reader = self.camera.as_ref().ok_or(AmmlStatus::NOT_AVAILABLE)?;
        // SAFETY: the dispatch runs on this thread inside the `renderLevel` call that was given these arguments, which
        // the game keeps alive until that call returns.
        unsafe { reader.read(CALL.with(Cell::get)) }.ok_or(AmmlStatus::FAILED)
    }

    /// The clip matrix of the camera of `frame`: projection times view, by columns. `NOT_AVAILABLE` without the area
    /// version 3, `REFUSED` outside the dispatch of `frame` on this thread, `FAILED` when the frame has no camera or a
    /// matrix stack is empty.
    pub fn canvas_clip_matrix(&self, frame: u64) -> Result<[[f32; 4]; 4], AmmlStatus> {
        self.check_area(Self::GHOST_VERSION)?;
        if !RenderScope::allows(frame) {
            return Err(AmmlStatus::REFUSED);
        }
        let parts = self.mesh.as_ref().ok_or(AmmlStatus::NOT_AVAILABLE)?;
        // SAFETY: the dispatch runs inside the `renderLevel` call whose screen context this is, on this thread.
        unsafe { parts.drawer.clip_matrix(CALL.with(Cell::get).screen) }.ok_or(AmmlStatus::FAILED)
    }

    /// Adds the game mesh at `handle` (a mesh the block mesher ended, made by `MeshBuilder`) to the mesh book as a
    /// block mesh of the mod at `owner`, and gives its id. It follows the rules of every mesh of the book: a free, a
    /// world leave, a device reset, a graphics mode switch and the disable of the mod end it, and its game object is
    /// destroyed on the `renderLevel` thread. It is drawn with the blend-block material and the terrain atlas.
    pub fn adopt_block_mesh(&self, owner: usize, handle: usize) -> AmmlMeshId {
        let key = self.meshes.create_block(owner, handle);
        AmmlMeshId { id: key.id, generation: key.generation, _reserved: 0 }
    }

    /// The checks of a mesher build of a grid of `dims` with `cell_count` cells in the frame `frame`, made before a cell is
    /// read: `NOT_AVAILABLE` without the area, `REFUSED` outside the dispatch of `frame`, off the game thread or for a
    /// side above the cache window, `INVALID_ARGUMENT` for a cell count other than the product of `dims`. Logs the thread
    /// of the calls once and on a change.
    pub fn check_mesher(&self, frame: u64, dims: [u32; 3], cell_count: usize) -> Result<(), AmmlStatus> {
        self.check_frame(frame)?;
        if dims.iter().any(|side| *side > CacheWindow::SIDE) {
            return Err(AmmlStatus::REFUSED);
        }
        if !CacheWindow::fits(dims, cell_count) {
            return Err(AmmlStatus::INVALID_ARGUMENT);
        }
        Ok(())
    }

    /// Builds the grid `cells` of `dims` at `origin` with the palette `palette` (block handles of the mod at `owner`) into a
    /// block mesh of that mod in the frame `frame`, and gives what was built and the grid indices of the skipped cells. See
    /// `RenderMesherBuildFn`.
    #[allow(clippy::too_many_arguments)]
    pub fn mesher_build(
        &self,
        owner: usize,
        frame: u64,
        dims: [u32; 3],
        origin: [i32; 3],
        cells: &[u16],
        palette: &[AmmlBlockHandle],
        options: &AmmlMesherOptions,
    ) -> Result<(AmmlMeshBuilt, Vec<u32>), AmmlStatus> {
        self.check_mesher(frame, dims, cells.len())?;
        if cells.iter().any(|cell| *cell != CacheWindow::EMPTY && usize::from(*cell) >= palette.len()) {
            return Err(AmmlStatus::INVALID_ARGUMENT);
        }
        let blocks =
            palette.iter().map(|handle| BlockHandles::address(self.state, *handle)).collect::<Result<Vec<_>, _>>()?;
        let parts = self.mesh.as_ref().ok_or(AmmlStatus::NOT_AVAILABLE)?;
        let mesher = self.mesher.as_ref().ok_or(AmmlStatus::NOT_AVAILABLE)?;
        let reads = BlockReadAdapter::get().ok_or(AmmlStatus::NOT_AVAILABLE)?;
        let source = ActorAdapter::get().ok_or(AmmlStatus::NOT_AVAILABLE)?.primary_block_source()?;
        let epoch = self.state.events().gate().epoch().0;
        // SAFETY: `source` is the live block source of the local player, read on the game thread inside the dispatch.
        let world = move |pos: [i32; 3]| unsafe { reads.block_address(source, pos) };
        let world_block: Option<&dyn Fn([i32; 3]) -> usize> = (options.skip_matching_world != 0).then_some(&world);
        let heap = MsvcHeap::new(ClientAdapter::heap_new, ClientAdapter::heap_delete);
        let call = CALL.with(Cell::get);
        // SAFETY: the dispatch runs inside the `renderLevel` call whose screen context this is, on the game thread
        // (checked); `source` is the live block source of the local player of this world; each palette block is a live
        // game block of this world (its handle is not stale); the heap is the game's allocator.
        let (handle, quads, skipped) = unsafe {
            mesher.build(
                &parts.builder,
                call.screen,
                source,
                epoch,
                (dims, origin, cells),
                &blocks,
                self.look(options),
                world_block,
                &heap,
            )
        }?;
        let mesh = handle.map(|handle| self.adopt_block_mesh(owner, handle)).unwrap_or_default();
        let skipped_count = u32::try_from(skipped.len()).map_err(|_| AmmlStatus::FAILED)?;
        Ok((AmmlMeshBuilt { mesh, quads, skipped_count }, skipped))
    }

    // The alpha of every vertex and whether water takes the biome colour: the option is on and the area is offered.
    fn look(&self, options: &AmmlMesherOptions) -> (u8, bool) {
        let water = options.water_tint != 0 && self.state.areas().is_available(Self::WATER_TINT, Self::WATER_VERSION);
        (options.alpha, water)
    }

    /// The checks of a block mesher call in the frame `frame`: `NOT_AVAILABLE` without the area, `REFUSED` outside the dispatch of
    /// `frame` or off the game thread. Logs the thread of the calls once and on a change.
    pub fn check_frame(&self, frame: u64) -> Result<(), AmmlStatus> {
        if !self.state.areas().is_available(Self::BLOCK_MESHER, Self::MESHER_VERSION) {
            return Err(AmmlStatus::NOT_AVAILABLE);
        }
        if !RenderScope::allows(frame) {
            return Err(AmmlStatus::REFUSED);
        }
        let game = self.state.events().gate().on_game_thread();
        if self.mesher_thread.changed(if game { AmmlThread::GAME } else { AmmlThread::RENDER }) {
            curia::info!("world thread", { hook: "BlockTessellator.tessellateInWorld", game: game });
        }
        if game { Ok(()) } else { Err(AmmlStatus::REFUSED) }
    }

    /// Opens a mesh group of the mod at `owner` whose slots are `slot_size` blocks. `INVALID_ARGUMENT` for a slot with a
    /// side of 0, `REFUSED` for the 65th open group of the mod (the slot size is checked and not otherwise used by the loader). See `RenderMesherGroupBeginFn`.
    pub fn group_begin(&self, owner: usize, frame: u64, slot_size: [u32; 3]) -> Result<AmmlMeshGroup, AmmlStatus> {
        self.check_frame(frame)?;
        if slot_size.contains(&0) {
            return Err(AmmlStatus::INVALID_ARGUMENT);
        }
        self.mesher.as_ref().ok_or(AmmlStatus::NOT_AVAILABLE)?.group_begin(owner)
    }

    /// Appends the grid `cells` of `dims` at `origin`, moved by `offset`, with the palette `palette` (block handles of the
    /// mod at `owner`), to the group `group` of that mod in the frame `frame`. With `skip_matching_world` the matching
    /// cells are left out as in a build; their indices are not reported. See `RenderMesherGroupAppendFn`.
    #[allow(clippy::too_many_arguments)]
    pub fn group_append(
        &self,
        owner: usize,
        frame: u64,
        group: AmmlMeshGroup,
        grid: ([u32; 3], [i32; 3], &[u16]),
        palette: &[AmmlBlockHandle],
        offset: [i32; 3],
        options: &AmmlMesherOptions,
    ) -> Result<(), AmmlStatus> {
        let (dims, _, cells) = grid;
        self.check_mesher(frame, dims, cells.len())?;
        let mesher = self.mesher.as_ref().ok_or(AmmlStatus::NOT_AVAILABLE)?;
        mesher.group_check(owner, group)?;
        if cells.iter().any(|cell| *cell != CacheWindow::EMPTY && usize::from(*cell) >= palette.len()) {
            return Err(AmmlStatus::INVALID_ARGUMENT);
        }
        let blocks =
            palette.iter().map(|handle| BlockHandles::address(self.state, *handle)).collect::<Result<Vec<_>, _>>()?;
        let parts = self.mesh.as_ref().ok_or(AmmlStatus::NOT_AVAILABLE)?;
        let reads = BlockReadAdapter::get().ok_or(AmmlStatus::NOT_AVAILABLE)?;
        let source = ActorAdapter::get().ok_or(AmmlStatus::NOT_AVAILABLE)?.primary_block_source()?;
        let epoch = self.state.events().gate().epoch().0;
        // SAFETY: `source` is the live block source of the local player, read on the game thread inside the dispatch.
        let world = move |pos: [i32; 3]| unsafe { reads.block_address(source, pos) };
        let world_block: Option<&dyn Fn([i32; 3]) -> usize> = (options.skip_matching_world != 0).then_some(&world);
        let heap = MsvcHeap::new(ClientAdapter::heap_new, ClientAdapter::heap_delete);
        let call = CALL.with(Cell::get);
        // SAFETY: as `mesher_build`; the group is open and of this mod and world (checked).
        unsafe {
            mesher.group_append(
                owner,
                group,
                &parts.builder,
                call.screen,
                source,
                epoch,
                grid,
                offset,
                &blocks,
                self.look(options),
                world_block,
                &heap,
            )
        }
    }

    /// Ends the group `group` of the mod at `owner` into one block mesh of that mod in the frame `frame`; the group is
    /// gone after the call. A group that holds nothing gives mesh id 0 and no quads. See `RenderMesherGroupFinishFn`.
    pub fn group_finish(&self, owner: usize, frame: u64, group: AmmlMeshGroup) -> Result<AmmlMeshBuilt, AmmlStatus> {
        self.check_frame(frame)?;
        let parts = self.mesh.as_ref().ok_or(AmmlStatus::NOT_AVAILABLE)?;
        let mesher = self.mesher.as_ref().ok_or(AmmlStatus::NOT_AVAILABLE)?;
        let heap = MsvcHeap::new(ClientAdapter::heap_new, ClientAdapter::heap_delete);
        let call = CALL.with(Cell::get);
        // SAFETY: the dispatch runs inside the `renderLevel` call whose screen context this is, on the game thread
        // (checked); the heap is the game's allocator.
        let (handle, quads) = unsafe { mesher.group_finish(owner, group, &parts.builder, call.screen, &heap) }?;
        let mesh = handle.map(|handle| self.adopt_block_mesh(owner, handle)).unwrap_or_default();
        Ok(AmmlMeshBuilt { mesh, quads, skipped_count: 0 })
    }
    /// Ends the group `group` of the mod at `owner` without a mesh, from any thread: the slot is free at once and the
    /// buffer is destroyed on the `renderLevel` thread at its next call. See `RenderMesherGroupDiscardFn`.
    pub fn group_discard(&self, owner: usize, group: AmmlMeshGroup) -> Result<(), AmmlStatus> {
        let discarded = self.mesher.as_ref().ok_or(AmmlStatus::NOT_AVAILABLE)?.group_discard(owner, group);
        self.wake();
        discarded
    }

    /// Frees the mesh `mesh` of the mod at `owner`. The id is stale at once; the game object is destroyed on the
    /// `renderLevel` thread at its next call.
    pub fn mesh_free(&self, owner: usize, mesh: AmmlMeshId) -> Result<(), AmmlStatus> {
        self.check_area(Self::CAMERA_VERSION)?;
        let freed = self.meshes.free(owner, Self::key(mesh)).map_err(Self::refusal);
        self.wake();
        freed
    }

    /// The checks of a build of `count` quads with `style` in the frame `frame`, made before any quad is read:
    /// `NOT_AVAILABLE` without the area version 2 or for a style this version does not draw, `REFUSED` outside the
    /// dispatch of `frame` or above `MAX_QUADS`, `INVALID_ARGUMENT` for no quads or a style byte out of range.
    pub fn check_build(&self, frame: u64, count: usize, style: AmmlMeshStyle) -> Result<(), AmmlStatus> {
        self.check_area(Self::MESH_VERSION)?;
        if !RenderScope::allows(frame) {
            return Err(AmmlStatus::REFUSED);
        }
        Self::check_style(style)?;
        if count == 0 {
            return Err(AmmlStatus::INVALID_ARGUMENT);
        }
        if count > Self::MAX_QUADS as usize {
            return Err(AmmlStatus::REFUSED);
        }
        Ok(())
    }

    /// Builds a mesh of `quads` with `style` for the mod at `owner` in the frame `frame`. See `RenderMeshBuildFn`.
    pub fn mesh_build(
        &self,
        owner: usize,
        frame: u64,
        quads: &[AmmlColorQuad],
        style: AmmlMeshStyle,
    ) -> Result<AmmlMeshId, AmmlStatus> {
        self.check_build(frame, quads.len(), style)?;
        let finite = |quad: &AmmlColorQuad| {
            quad.corners.iter().all(|at| at.x.is_finite() && at.y.is_finite() && at.z.is_finite())
        };
        if !quads.iter().all(finite) {
            return Err(AmmlStatus::INVALID_ARGUMENT);
        }
        let parts = self.mesh.as_ref().ok_or(AmmlStatus::NOT_AVAILABLE)?;
        let call = CALL.with(Cell::get);
        let heap = MsvcHeap::new(ClientAdapter::heap_new, ClientAdapter::heap_delete);
        // The second winding is written only for the name tag material, which hides back faces; the Vibrant Visuals
        // wireframe material shows both sides. A mode switch frees every mesh, so the mode now is the mode of its draws.
        let two_sided = style.two_sided != 0 && !self.vibrant.load(Ordering::Acquire);
        // SAFETY: the dispatch runs inside the `renderLevel` call whose screen context this is, on this thread; the
        // heap is the game's allocator.
        let handle = unsafe { parts.builder.build(call.screen, quads, two_sided, &heap) }?;
        let key = self.meshes.create(owner, handle);
        Ok(AmmlMeshId { id: key.id, generation: key.generation, _reserved: 0 })
    }

    /// Draws the mesh `mesh` of the mod at `owner` with `transform` in the frame `frame`. See `RenderMeshDrawFn`.
    pub fn mesh_draw(
        &self,
        owner: usize,
        frame: u64,
        mesh: AmmlMeshId,
        transform: &AmmlTransform,
    ) -> Result<(), AmmlStatus> {
        self.check_area(Self::MESH_VERSION)?;
        if !RenderScope::allows(frame) {
            return Err(AmmlStatus::REFUSED);
        }
        let offset = transform.offset;
        let numbers = [offset.x, offset.y, offset.z, transform.scale, transform.yaw_degrees];
        if !numbers.iter().all(|value| value.is_finite()) {
            return Err(AmmlStatus::INVALID_ARGUMENT);
        }
        let parts = self.mesh.as_ref().ok_or(AmmlStatus::NOT_AVAILABLE)?;
        let (handle, kind) = self.meshes.entry(owner, Self::key(mesh)).map_err(Self::refusal)?;
        let camera = self.frame_camera()?;
        let origin = transform.origin;
        let placement = WorldPlacement {
            origin: [origin.x, origin.y, origin.z],
            offset: [offset.x, offset.y, offset.z],
            scale: transform.scale,
            yaw_degrees: transform.yaw_degrees,
        };
        let matrix = WorldTransform::matrix(&placement, camera);
        match kind {
            MeshKind::Colour => self.draw_colour(parts, owner, handle, &matrix),
            MeshKind::Block => self.draw_block(parts, handle, &matrix),
        }
    }

    // A colour mesh: the world material of the graphics mode and the loader's white texture.
    fn draw_colour(
        &self,
        parts: &MeshParts,
        owner: usize,
        handle: usize,
        matrix: &[[f32; 4]; 4],
    ) -> Result<(), AmmlStatus> {
        let texture = TextureAdapter::with(None, |textures| Some(textures.white_texture()))
            .ok_or(AmmlStatus::NOT_AVAILABLE)?
            .ok_or(AmmlStatus::NOT_READY)?;
        let call = CALL.with(Cell::get);
        let vibrant = self.vibrant.load(Ordering::Acquire);
        // SAFETY: the dispatch runs inside the `renderLevel` call of this renderer, on this thread.
        let (choice, wireframe) = unsafe { parts.material.prepare(owner, vibrant, call.renderer) };
        RenderLevelDetours::set(RenderLevelDetours::MATERIAL, parts.material.is_applied());
        let material = match choice {
            MaterialChoice::Wireframe => wireframe,
            // SAFETY: the game heap's allocator; render thread, inside the `renderLevel` call.
            MaterialChoice::NameTag => unsafe { parts.name_tag.material(ClientAdapter::heap_new) },
        };
        let Some(material) = material else {
            let mut texture = texture;
            // SAFETY: the copy holds the uses `white_texture` took; nothing else holds them.
            unsafe { texture.release() };
            return Err(AmmlStatus::FAILED);
        };
        // SAFETY: the screen context of the running call, a live mesh of this book, a live material and a copy of the
        // white texture that holds its own uses.
        unsafe { parts.drawer.draw(call.screen, handle, material, texture, matrix) }
    }

    // A block mesh: the client's blend-block material and a copy of the renderer's terrain atlas, in every graphics
    // mode. The material is read before the atlas is copied, so a refusal holds no use of the atlas.
    fn draw_block(&self, parts: &MeshParts, handle: usize, matrix: &[[f32; 4]; 4]) -> Result<(), AmmlStatus> {
        self.check_area(Self::GHOST_VERSION)?;
        let game = self.state.events().gate().on_game_thread();
        if self.ghost_thread.changed(if game { AmmlThread::GAME } else { AmmlThread::RENDER }) {
            curia::info!("world thread", { hook: "ClientInstance.getItemInHandRenderer", game: game });
        }
        let client = SessionAdapter::primary_client().ok_or(AmmlStatus::FAILED)?;
        // SAFETY: the primary client is live while a world is joined; the dispatch runs on the game thread on both gate
        // builds (`world thread` above says so at run time).
        let material = unsafe { parts.ghost.blend_material(client) }.ok_or(AmmlStatus::FAILED)?;
        let call = CALL.with(Cell::get);
        // SAFETY: the dispatch runs inside the `renderLevel` call of this renderer, on this thread.
        let texture = unsafe { parts.ghost.atlas_copy(call.renderer) }.ok_or(AmmlStatus::NOT_READY)?;
        // SAFETY: the screen context of the running call, a live mesh of this book, a live material and a copy of the
        // atlas that holds its own uses.
        unsafe { parts.drawer.draw(call.screen, handle, material, texture, matrix) }
    }

    // A build from colour quads gives the one overlay style: alpha blend, depth test, no depth write, full light, the
    // white texture. A block mesh (blend-block material, terrain atlas) comes only from the block mesher: colour quads
    // carry no UV, so the ghost style is `NOT_AVAILABLE` here.
    fn check_style(style: AmmlMeshStyle) -> Result<(), AmmlStatus> {
        let flags = [style.blend, style.depth_test, style.depth_write, style.two_sided, style.light, style.texture];
        if flags.iter().any(|value| *value > 1) || style._reserved != [0; 2] {
            return Err(AmmlStatus::INVALID_ARGUMENT);
        }
        let overlay = style.blend == AmmlMeshStyle::BLEND_ALPHA && style.depth_test == 1 && style.depth_write == 0;
        if !overlay || style.light != AmmlMeshStyle::LIGHT_FULL || style.texture != AmmlMeshStyle::TEXTURE_WHITE {
            return Err(AmmlStatus::NOT_AVAILABLE);
        }
        Ok(())
    }

    fn key(mesh: AmmlMeshId) -> MeshKey {
        MeshKey { id: mesh.id, generation: mesh.generation }
    }

    fn refusal(refusal: MeshRefusal) -> AmmlStatus {
        match refusal {
            MeshRefusal::Stale => AmmlStatus::STALE,
            MeshRefusal::NotOwner => AmmlStatus::REFUSED,
        }
    }
}
