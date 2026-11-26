use std::cell::Cell;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU64, Ordering};

use amml_abi::{
    AmmlBlockEntityCamera, AmmlBlockEntityGhost, AmmlBlockHandle, AmmlEvent, AmmlGhostEntityOptions, AmmlPhase,
    AmmlStatus, AmmlThread, BlockEntityPass,
};
use amml_data::{AdapterCatalog, Availability, EntryId, NeedKind};
use amml_msvc::RefCountBase;
use amml_runtime::{GhostEntity, GhostEntityBook, GhostEntityKey, GhostEntityRefusal};

use crate::adapters::BlockEntityCalls;
use crate::{
    ActorAdapter, BlockEntityDetours, BlockHandles, ChestPairWriter, ClientAdapter, EventHub, InventoryAdapter,
    LoaderState, ObjectRead, RenderLevelDetours, SessionAdapter, SkullDrawShape, StringByteInputLayout, ThreadNote,
};

static ADAPTER: OnceLock<BlockEntityAdapter> = OnceLock::new();

/// `IVanillaRenderBlockActorComponent* BlockActor::_getRenderComponent()`: rcx the actor.
type RenderComponentFn = unsafe extern "C" fn(usize) -> usize;

thread_local! {
    // The frame id and the `BaseActorRenderContext` of the `BlockEntityPass` dispatch running on this thread; zero
    // outside.
    static PASS: Cell<(u64, usize)> = const { Cell::new((0, 0)) };
}

/// The block entity ghosts: raises `BlockEntityPass` after the game's opaque block entity pass of a level renderer
/// camera, and owns the loader's game block actors (`render.block_entities`, `render.chest_pair`,
/// `render.skull_pose`).
///
/// A ghost is the game's own block actor of a block, made by the factory or loaded with `loadStatic` from the mod's NBT,
/// held through the `shared_ptr` the game gave, in a `GhostEntityBook` bound to the mod and the world. It is drawn only
/// inside a pass dispatch, with its actor moved to the draw position (`BlockActor.mPosition`, the partner of a paired chest
/// with it), the dispatcher of the pass context, the block source of the local player, an empty
/// `MaterialPtr` (16 zero bytes, as the game's own callers pass a `.bss` static: survey row G4), no texture, no breaking
/// and a disengaged `optional<RenderMetadata>` at the derived flag. A ghost with a skull pose is drawn by the dispatcher's
/// skull renderer instead (`SkullDrawShape`), with the pose given at `create`. Ghosts end at `free`, the world leave, the
/// mod's
/// disable and a render device reset or surface loss; their actors are released on the `renderLevel` thread (at once
/// for a world leave on the game thread), each once.
pub struct BlockEntityAdapter {
    state: &'static LoaderState,
    ghosts: GhostEntityBook,
    calls: Option<BlockEntityCalls>,
    chest: Option<ChestPairWriter>,
    skull: Option<SkullDrawShape>,
    // The address of `LevelRendererPlayer.vtable`, the local player's level renderer camera.
    player_vtable: Option<usize>,
    frames: AtomicU64,
    pass_thread: ThreadNote,
    create_thread: ThreadNote,
}

impl BlockEntityAdapter {
    pub const BLOCK_ENTITIES: &'static str = "render.block_entities";
    pub const CHEST_PAIR: &'static str = "render.chest_pair";
    pub const SKULL_POSE: &'static str = "render.skull_pose";
    const VERSION: u32 = 1;
    // The size of a `CompoundTag`, the block `NbtIo::read` allocates for the tag it gives.
    const TAG_SIZE: usize = 0x18;
    // The position a ghost's actor is made at; each draw moves the actor to the world position it draws at (`place`).
    const ORIGIN: [i32; 3] = [0, 0, 0];
    // The highest face the game numbers (`AmmlFace::EAST`) and the highest turn of a skull, in sixteenths.
    const MAX_FACE: u8 = 5;
    const MAX_TURN: u8 = 15;

    /// Hooks `LevelRendererCamera.renderBlockEntitiesBody` and offers `render.block_entity_pass`,
    /// `render.block_entities`, `render.chest_pair` and `render.skull_pose` when their rows resolved. The ghost area
    /// also needs the actor adapter, the shared `renderLevel` detour (which releases the actors) and a
    /// `RenderMetadata.engaged` inside the loader's optional buffer; the chest and skull areas need the ghost area. Logs
    /// one `event offered` / `area adapter` or `... not available` line per row. Works once per process.
    pub fn install(state: &'static LoaderState) -> Vec<Availability> {
        let resolved = |name: &str| state.table().get(&EntryId(name.to_string())).is_some();
        let mut rows: Vec<Availability> =
            AdapterCatalog::availability(&resolved).into_iter().filter(Self::is_own_row).collect();
        let calls = Self::calls_for(state, &mut rows);
        let chest = Self::chest_for(state, &mut rows, calls.is_some());
        let skull = Self::skull_for(state, &mut rows, calls.is_some());
        let hooked = rows.iter().any(|row| row.shape.is_some()) && BlockEntityDetours::install(state);
        let adapter = Self {
            state,
            ghosts: GhostEntityBook::new(),
            calls,
            chest,
            skull,
            player_vtable: state
                .resolve("LevelRendererPlayer.vtable")
                .ok()
                .and_then(|value| usize::try_from(value).ok()),
            frames: AtomicU64::new(0),
            pass_thread: ThreadNote::new(),
            create_thread: ThreadNote::new(),
        };
        if ADAPTER.set(adapter).is_err() {
            curia::warn!("the block entity adapter is already installed");
            return rows;
        }
        for row in &mut rows {
            if row.shape.is_some() && !hooked {
                row.shape = None;
                row.missing = vec![BlockEntityDetours::ENTRY];
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
    pub fn get() -> Option<&'static BlockEntityAdapter> {
        ADAPTER.get()
    }

    /// Runs `body` on the installed adapter; `fallback` before `install`.
    pub(crate) fn with<R>(fallback: R, body: impl FnOnce(&BlockEntityAdapter) -> R) -> R {
        match ADAPTER.get() {
            Some(adapter) => body(adapter),
            None => fallback,
        }
    }

    fn is_own_row(row: &Availability) -> bool {
        match row.kind {
            NeedKind::Event => row.id == BlockEntityPass::ID,
            NeedKind::Area => {
                row.id == Self::BLOCK_ENTITIES || row.id == Self::CHEST_PAIR || row.id == Self::SKULL_POSE
            }
        }
    }

    // The ghost calls when the ghost row resolved and everything it rests on is there; else the row loses its shape and
    // names what is missing.
    fn calls_for(state: &'static LoaderState, rows: &mut [Availability]) -> Option<BlockEntityCalls> {
        let row = rows.iter_mut().find(|row| row.kind == NeedKind::Area && row.id == Self::BLOCK_ENTITIES)?;
        row.shape?;
        let made = BlockEntityCalls::new(state);
        match made {
            Ok(calls) => Some(calls),
            Err(missing) => {
                row.shape = None;
                row.missing = vec![missing];
                None
            }
        }
    }

    // The chest pair writer when its row resolved and the ghost area is there; else the row loses its shape.
    fn chest_for(state: &LoaderState, rows: &mut [Availability], ghosts: bool) -> Option<ChestPairWriter> {
        let row = rows.iter_mut().find(|row| row.kind == NeedKind::Area && row.id == Self::CHEST_PAIR)?;
        row.shape?;
        let writer = if ghosts { ChestPairWriter::new(state) } else { None };
        if writer.is_none() {
            row.shape = None;
            row.missing = vec![Self::BLOCK_ENTITIES];
        }
        writer
    }

    // The skull draw of the shape its row took when the ghost area is there; else the row loses its shape and names
    // what is missing.
    fn skull_for(state: &LoaderState, rows: &mut [Availability], ghosts: bool) -> Option<SkullDrawShape> {
        let row = rows.iter_mut().find(|row| row.kind == NeedKind::Area && row.id == Self::SKULL_POSE)?;
        let shape = row.shape?;
        let made = if ghosts { SkullDrawShape::new(state, shape) } else { Err(Self::BLOCK_ENTITIES) };
        match made {
            Ok(skull) => Some(skull),
            Err(missing) => {
                row.shape = None;
                row.missing = vec![missing];
                None
            }
        }
    }

    fn hub(&self) -> &EventHub {
        self.state.events()
    }

    /// After the game's opaque block entity pass of the camera at `camera` with the context at `context`: raises
    /// `BlockEntityPass` on this thread while a world is joined. `camera` is `PLAYER` when the camera's vtable is
    /// `LevelRendererPlayer.vtable`. The `frame_id` is new for each dispatch; no listener changes it or `camera`.
    pub(crate) fn pass(&self, camera: usize, context: usize) {
        let hub = self.hub();
        let gate = hub.gate();
        if !gate.is_open() || gate.epoch().0 == 0 || camera == 0 {
            return;
        }
        let game = gate.on_game_thread();
        if self.pass_thread.changed(if game { AmmlThread::GAME } else { AmmlThread::RENDER }) {
            curia::info!("world thread", { hook: BlockEntityDetours::ENTRY, game: game });
        }
        // SAFETY: `camera` is the live level renderer camera the game passed to the pass that just ran; its first qword
        // is its vtable pointer.
        let vtable = unsafe { ObjectRead::value::<usize>(camera, 0) };
        let kind = if vtable.is_some() && vtable == self.player_vtable {
            AmmlBlockEntityCamera::PLAYER
        } else {
            AmmlBlockEntityCamera::OTHER
        };
        let frame_id = self.frames.fetch_add(1, Ordering::Relaxed) + 1;
        let mut event = BlockEntityPass {
            header: EventHub::header::<BlockEntityPass>(AmmlPhase::AFTER),
            frame_id,
            camera: kind,
            _reserved: [0; 7],
        };
        let mut event_context = hub.context_on(if game { AmmlThread::GAME } else { AmmlThread::RENDER });
        if game && let Some(client) = SessionAdapter::primary_client() {
            event_context.client = std::ptr::with_exposed_provenance(client);
        }
        let keep = |event: &mut BlockEntityPass| {
            event.frame_id = frame_id;
            event.camera = kind;
            event._reserved = [0; 7];
        };
        let previous = PASS.with(|cell| cell.replace((frame_id, context)));
        hub.raise_keeping(&mut event, &event_context, &keep);
        PASS.with(|cell| cell.set(previous));
    }

    /// Makes a ghost of the block `block` of the mod at `owner`. See `RenderBlockEntityCreateFn`.
    pub fn create(
        &self,
        owner: usize,
        block: AmmlBlockHandle,
        nbt: &[u8],
        options: &AmmlGhostEntityOptions,
    ) -> Result<AmmlBlockEntityGhost, AmmlStatus> {
        self.check_area()?;
        self.enter_create()?;
        if options._reserved != [0; 4] || options.has_chest_pair > 1 || options.has_skull > 1 {
            return Err(AmmlStatus::INVALID_ARGUMENT);
        }
        let pair = if options.has_chest_pair == 1 {
            let pair = options.chest_pair;
            let along = matches!((pair.x, pair.y, pair.z), (-1 | 1, 0, 0) | (0, 0, -1 | 1));
            if !along {
                return Err(AmmlStatus::INVALID_ARGUMENT);
            }
            if !self.state.areas().is_available(Self::CHEST_PAIR, Self::VERSION) {
                return Err(AmmlStatus::NOT_AVAILABLE);
            }
            Some([pair.x, pair.y, pair.z])
        } else {
            None
        };
        let skull = if options.has_skull == 1 {
            let pose = options.skull;
            if pose.face.0 > Self::MAX_FACE || pose.turn > Self::MAX_TURN {
                return Err(AmmlStatus::INVALID_ARGUMENT);
            }
            if !self.state.areas().is_available(Self::SKULL_POSE, Self::VERSION) {
                return Err(AmmlStatus::NOT_AVAILABLE);
            }
            Some((pose.face.0, pose.turn))
        } else {
            None
        };
        let address = BlockHandles::address(self.state, block)?;
        if !self.ghosts.has_room(owner) {
            return Err(AmmlStatus::REFUSED);
        }
        let calls = self.calls.as_ref().ok_or(AmmlStatus::NOT_AVAILABLE)?;
        // SAFETY: `address` is a live block of this world (its handle is not stale); mBlockType is a pointer member.
        let block_type = unsafe { ObjectRead::pointer(address, calls.block_type_at) }.ok_or(AmmlStatus::FAILED)?;
        let level = self.level(calls)?;
        // SAFETY: a live block type and the live level of the local player, on the game thread (checked).
        let actor = unsafe { self.make_actor(calls, block_type, level, nbt) }?;
        if let Some(offset) = pair {
            let writer = self.chest.as_ref().ok_or(AmmlStatus::NOT_AVAILABLE)?;
            // SAFETY: a live actor the loader holds a use of; game thread.
            if !unsafe { writer.is_chest(actor[0]) } {
                Self::release(actor);
                return Err(AmmlStatus::INVALID_ARGUMENT);
            }
            // SAFETY: the game's own type test says the actor is a ChestBlockActor; game thread.
            unsafe { writer.write(actor[0], offset) };
        }
        let chest_paired = pair.is_some();
        if skull.is_some() {
            let Some(shape) = self.skull.as_ref() else {
                self.release_made(actor, chest_paired);
                return Err(AmmlStatus::NOT_AVAILABLE);
            };
            // SAFETY: a live actor the loader holds a use of; game thread.
            if !unsafe { shape.is_skull(actor[0]) } {
                self.release_made(actor, chest_paired);
                return Err(AmmlStatus::INVALID_ARGUMENT);
            }
        }
        let ghost = GhostEntity { actor: actor[0], control: actor[1], block: address, skull, chest_paired };
        match self.ghosts.create(owner, ghost) {
            Ok(key) => Ok(AmmlBlockEntityGhost { id: key.id, generation: key.generation }),
            Err(refusal) => {
                self.release_made(actor, chest_paired);
                Err(Self::refusal(refusal))
            }
        }
    }

    // The level of the local player of the primary client (`Actor.mLevel`).
    fn level(&self, calls: &BlockEntityCalls) -> Result<usize, AmmlStatus> {
        let player = ClientAdapter::get().and_then(ClientAdapter::primary_local_player).ok_or(AmmlStatus::NOT_READY)?;
        // SAFETY: the live local player of the primary client, on the game thread; mLevel is a pointer member.
        unsafe { ObjectRead::pointer(player, calls.level_at) }.ok_or(AmmlStatus::FAILED)
    }

    // The game's actor of `block_type`: loaded from `nbt` with `loadStatic` when it holds bytes, else made by the
    // factory. The tag the read made is destroyed and freed (18h bytes on the game heap) before this returns.
    //
    // Safety: a live block type and the live level of the local player; game thread.
    unsafe fn make_actor(
        &self,
        calls: &BlockEntityCalls,
        block_type: usize,
        level: usize,
        nbt: &[u8],
    ) -> Result<[usize; 2], AmmlStatus> {
        if nbt.is_empty() {
            // SAFETY: forwarded from the caller.
            return unsafe { calls.factory.make(block_type, Self::ORIGIN, level) }.ok_or(AmmlStatus::FAILED);
        }
        let mut input = StringByteInputLayout::new(calls.input_vtable, nbt);
        let mut tag = 0usize;
        // SAFETY: an input over `nbt`, which outlives the call, and an 8-byte result.
        unsafe { (calls.nbt_read)(&mut tag, &mut input) };
        if tag == 0 {
            return Err(AmmlStatus::FAILED);
        }
        // The `DefaultDataLoadHelper` is only its vtable pointer, as the game builds it on the stack (survey row E3).
        let mut helper = calls.helper_vtable;
        let mut out = [0usize; 2];
        // SAFETY: a 16-byte result, the caller's live block type and level, a position, the live tag and a helper that
        // live for the call.
        unsafe { (calls.load_static)(&mut out, block_type, &Self::ORIGIN, level, tag, &mut helper) };
        // SAFETY: the tag is the game's, made by the read above, and is destroyed once; then its block is freed with
        // the size of a CompoundTag, as its deleting destructor does.
        unsafe {
            (calls.tag_dtor)(tag);
            self.state.heap().free(std::ptr::with_exposed_provenance_mut(tag), Self::TAG_SIZE);
        }
        if out[0] == 0 {
            Self::release(out);
            return Err(AmmlStatus::FAILED);
        }
        Ok(out)
    }

    /// Frees the ghost `ghost` of the mod at `owner`; its actor is released at the next `renderLevel`.
    pub fn free(&self, owner: usize, ghost: AmmlBlockEntityGhost) -> Result<(), AmmlStatus> {
        self.check_area()?;
        let freed = self.ghosts.free(owner, Self::key(ghost)).map_err(Self::refusal);
        self.wake();
        freed
    }

    /// Draws the ghost `ghost` of the mod at `owner` at the world position `pos` in the pass `frame`. See
    /// `RenderBlockEntityDrawFn`.
    pub fn draw(&self, owner: usize, frame: u64, ghost: AmmlBlockEntityGhost, pos: [i32; 3]) -> Result<(), AmmlStatus> {
        self.check_area()?;
        let (current, context) = PASS.with(Cell::get);
        if frame == 0 || frame != current || context == 0 {
            return Err(AmmlStatus::REFUSED);
        }
        let entry = self.ghosts.entry(owner, Self::key(ghost)).map_err(Self::refusal)?;
        let calls = self.calls.as_ref().ok_or(AmmlStatus::NOT_AVAILABLE)?;
        // SAFETY: the context of the running pass, which the game keeps alive for the pass; pointer members.
        let dispatcher = unsafe { ObjectRead::pointer(context, calls.dispatcher_at) }.ok_or(AmmlStatus::FAILED)?;
        // SAFETY: as above.
        let implementation = unsafe { ObjectRead::pointer(context, calls.impl_at) }.ok_or(AmmlStatus::FAILED)?;
        // SAFETY: the live `Impl` of the context; the camera target is a Vec3 member.
        let camera = unsafe { ObjectRead::value::<[f32; 3]>(implementation, calls.camera_target_at) }
            .ok_or(AmmlStatus::FAILED)?;
        let render_pos = [pos[0] as f32 - camera[0], pos[1] as f32 - camera[1], pos[2] as f32 - camera[2]];
        let mut metadata = calls.metadata.disengaged();
        if let Some(pose) = entry.skull {
            let shape = self.skull.as_ref().ok_or(AmmlStatus::NOT_AVAILABLE)?;
            // SAFETY: a live block of this world (the ghost is not stale); mBlockType is a pointer member.
            let block_type =
                unsafe { ObjectRead::pointer(entry.block, calls.block_type_at) }.ok_or(AmmlStatus::FAILED)?;
            // SAFETY: the context and dispatcher of the running pass on its thread, a live block type, and zero bytes
            // that cover a `RenderMetadata` of either build (the optional buffer holds one before its engaged byte).
            return unsafe { shape.draw(context, dispatcher, block_type, pose, render_pos, metadata.as_ptr()) };
        }
        let function =
            InventoryAdapter::virtual_fn(entry.actor, calls.render_component_slot).ok_or(AmmlStatus::FAILED)?;
        // SAFETY: the slot of `_getRenderComponent` in the vtable of a live actor the loader holds.
        let component_of = unsafe { std::mem::transmute::<usize, RenderComponentFn>(function) };
        // SAFETY: called on its own live actor; it gives an interior pointer or null.
        let component = unsafe { component_of(entry.actor) };
        if component == 0 {
            return Err(AmmlStatus::FAILED);
        }
        let source = ActorAdapter::get().ok_or(AmmlStatus::NOT_AVAILABLE)?.primary_block_source()?;
        // SAFETY: a live actor the loader holds, drawn on this thread only; a chest it paired is a ChestBlockActor.
        unsafe { self.place(calls, &entry, pos) };
        let material = [0u64; 2];
        // SAFETY: the dispatcher and the context of the running pass, the local player's live block source, the live
        // actor's component, a live block, and buffers that live for the call: both positions, an empty `MaterialPtr`,
        // no texture, and a disengaged optional that the callee destroys by reading its engaged byte only.
        unsafe {
            (calls.render)(
                dispatcher,
                context,
                source,
                component,
                entry.block,
                &render_pos,
                &pos,
                0,
                &material,
                0,
                0,
                metadata.as_mut_ptr(),
            )
        };
        Ok(())
    }

    // Moves the actor of `entry` to `pos`, the place it is drawn at: the game's bed, banner and sign text renderers read
    // the light at the actor's own `mPosition`, not at the draw position (26.63 0x6af9360, 0x6af1be0, 0x6a7ffd0; 70.26
    // 0x8a29440). The partner of a paired chest moves with it.
    //
    // Safety: `entry.actor` is a live actor the loader holds, drawn on this thread only; when `entry.chest_paired` it
    // is a ChestBlockActor that `ChestPairWriter::write` paired.
    unsafe fn place(&self, calls: &BlockEntityCalls, entry: &GhostEntity, pos: [i32; 3]) {
        let at = std::ptr::with_exposed_provenance_mut::<[i32; 3]>(entry.actor + calls.position_at);
        // SAFETY: `mPosition` of a live actor (`BlockActor.mPosition`).
        let from = unsafe { at.read_unaligned() };
        if from == pos {
            return;
        }
        if entry.chest_paired
            && let Some(writer) = self.chest.as_ref()
        {
            // SAFETY: forwarded from the caller.
            unsafe { writer.move_partner(entry.actor, from, pos) };
        }
        // SAFETY: as above.
        unsafe { at.write_unaligned(pos) };
    }

    /// The number of live ghosts.
    pub fn live_ghosts(&self) -> usize {
        self.ghosts.len()
    }

    /// Before `WorldLeft`: every ghost is gone and its id stale. On the game thread the actors are released at once,
    /// else at the next `renderLevel`.
    pub(crate) fn world_left(&self) {
        let count = self.ghosts.world_left();
        if count > 0 {
            curia::info!("ghost entities freed", { reason: "world_left", count: count });
        }
        if self.hub().gate().on_game_thread() {
            self.destroy_due();
        }
        self.wake();
    }

    /// The surface is lost or the render device was reset: every ghost is gone; the actors wait for `renderLevel`.
    pub(crate) fn device_reset(&self) {
        let count = self.ghosts.device_reset();
        if count > 0 {
            curia::info!("ghost entities freed", { reason: "device_reset", count: count });
        }
        self.wake();
    }

    /// The mod at `owner` is disabled: its ghosts are gone; the actors wait for `renderLevel`.
    pub(crate) fn disable_owner(&self, owner: usize) {
        let count = self.ghosts.disable_owner(owner);
        if count > 0 {
            curia::info!("ghost entities freed", { reason: "disable", count: count });
        }
        self.wake();
    }

    /// Before the game's `renderLevel`, on its thread: the actors that wait are released, each once.
    pub(crate) fn destroy_due(&self) {
        let due = self.ghosts.take_due();
        for ghost in &due {
            self.release_made([ghost.actor, ghost.control], ghost.chest_paired);
        }
        if !due.is_empty() {
            curia::debug!("ghost entities released", { count: due.len() });
        }
    }

    // Tells the render detour whether an actor waits for its thread.
    fn wake(&self) {
        if self.ghosts.has_due() {
            RenderLevelDetours::set(RenderLevelDetours::GHOST_DUE, true);
        }
    }

    // Drops the loader's use of an actor it made; a chest it paired is unpaired first (`ChestPairWriter::unpair`).
    fn release_made(&self, actor: [usize; 2], chest_paired: bool) {
        if chest_paired && let Some(writer) = self.chest.as_ref() {
            // SAFETY: the loader paired this ChestBlockActor and still holds its use; the release runs on the game or
            // `renderLevel` thread.
            unsafe { writer.unpair(actor[0]) };
        }
        Self::release(actor);
    }

    // Drops the use of the `shared_ptr` (actor, control block) the loader holds, as its destructor does.
    fn release(actor: [usize; 2]) {
        if actor[1] != 0 {
            // SAFETY: the control block of a `shared_ptr` the game gave the loader with one use, which the loader owns
            // and gives back here once.
            unsafe { RefCountBase::decref(std::ptr::with_exposed_provenance_mut(actor[1])) };
        }
    }

    fn check_area(&self) -> Result<(), AmmlStatus> {
        if self.state.areas().is_available(Self::BLOCK_ENTITIES, Self::VERSION) {
            Ok(())
        } else {
            Err(AmmlStatus::NOT_AVAILABLE)
        }
    }

    // The gate, a joined world, then the game thread; logs the thread of the calls once and on a change.
    fn enter_create(&self) -> Result<(), AmmlStatus> {
        let gate = self.hub().gate();
        gate.require_open()?;
        let game = gate.on_game_thread();
        if self.create_thread.changed(if game { AmmlThread::GAME } else { AmmlThread::RENDER }) {
            curia::info!("world thread", { hook: "BlockActor.loadStatic", game: game });
        }
        if !game {
            return Err(AmmlStatus::REFUSED);
        }
        if gate.epoch().0 == 0 {
            return Err(AmmlStatus::NOT_READY);
        }
        Ok(())
    }

    fn key(ghost: AmmlBlockEntityGhost) -> GhostEntityKey {
        GhostEntityKey { id: ghost.id, generation: ghost.generation }
    }

    fn refusal(refusal: GhostEntityRefusal) -> AmmlStatus {
        match refusal {
            GhostEntityRefusal::Stale => AmmlStatus::STALE,
            GhostEntityRefusal::NotOwner | GhostEntityRefusal::TooMany => AmmlStatus::REFUSED,
        }
    }
}
