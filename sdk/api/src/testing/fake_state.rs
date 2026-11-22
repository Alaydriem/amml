use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::atomic::{AtomicU32, Ordering};

use amml_abi::{AmmlDirKind, AmmlLogLevel, AmmlStatus, AmmlStr, KnownLogLevel};
use amml_runtime::{EventBus, ListenError, ListenerId, ProviderRegistry};

use crate::Area;
use crate::testing::{
    FakeCameraFree, FakeCameraState, FakeChatBook, FakeClient, FakeInput, FakeInventory, FakeItemBook, FakeLighting,
    FakeListener, FakeNetwork, FakeOptionsState, FakeOverlay, FakePhase, FakeRender, FakeRequests, FakeShapes,
    FakeTextureBook, FakeTime, FakeUi, FakeWindow,
};

thread_local! {
    // The fake loader the table functions of this thread call into.
    static CURRENT: RefCell<Option<Rc<FakeState>>> = const { RefCell::new(None) };
}

// The tag of the next fake loader; unique in the process, never 0.
static NEXT_TAG: AtomicU32 = AtomicU32::new(1);

/// The state of one fake loader: the event bus and provider registry of `amml-runtime`, the gate,
/// the epoch, what the mod logged, the area book, the client, the input service, the window and the
/// overlay.
pub(crate) struct FakeState {
    // In the high 32 bits of every listener handle, so a handle of another fake is refused.
    tag: u32,
    bus: EventBus<FakeListener>,
    providers: ProviderRegistry<usize>,
    gate: Cell<bool>,
    epoch: Cell<u64>,
    logs: RefCell<Vec<String>>,
    missing_required: RefCell<Vec<String>>,
    // Events whose listens give `REFUSED`.
    refused: RefCell<Vec<&'static str>>,
    // Never removed, so a view handed out by `dir` stays valid while the state lives.
    dirs: RefCell<Vec<(AmmlDirKind, Box<str>)>>,
    phase: Cell<FakePhase>,
    areas: RefCell<Vec<(&'static str, u32)>>,
    client: RefCell<Option<FakeClient>>,
    // The chunk positions the fake gave handles for, in the order of their handles.
    chunks: RefCell<Vec<(i32, i32)>>,
    // The block positions the fake gave block handles for, in the order of their handles.
    blocks: RefCell<Vec<crate::BlockPos>>,
    // The token and the new state of the `BlockChanged` event that runs; `None` outside the raise.
    changed: Cell<Option<(u32, crate::BlockHandle)>>,
    last_changed_token: Cell<u32>,
    inventory: Cell<FakeInventory>,
    lighting: FakeLighting,
    client_writes: RefCell<Vec<String>>,
    rotation_writes: RefCell<Vec<(crate::Rotation, Option<f32>)>>,
    // Each `GameMode::build_at` the game took: `build_at x,y,z face=up`.
    builds: RefCell<Vec<String>>,
    // Each `Blocks::rotated` call: the handle and the quarter count after the modulo.
    block_rotations: RefCell<Vec<(crate::BlockHandle, u8)>>,
    // The id the next `join_server` gives.
    next_join: Cell<u64>,
    // A focus loss a listener cancelled and nothing has replayed or ended; the fake has one mod.
    focus_held: Cell<bool>,
    focus_replays: Cell<u32>,
    input: FakeInput,
    window: Cell<Option<FakeWindow>>,
    window_requests: RefCell<Vec<String>>,
    overlay: FakeOverlay,
    ui: FakeUi,
    textures: FakeTextureBook,
    chat: FakeChatBook,
    items: FakeItemBook,
    requests: FakeRequests,
    network: FakeNetwork,
    render: FakeRender,
    block_entities: crate::testing::FakeBlockEntities,
    camera: FakeCameraState,
    camera_free: FakeCameraFree,
    options: FakeOptionsState,
    shapes: FakeShapes,
    time: FakeTime,
}

impl FakeState {
    /// The owner every listener and table of the fake belongs to.
    const OWNER: usize = 1;
    /// The bus id part of a handle.
    const ID_MASK: u64 = 0xFFFF_FFFF;

    /// Where the fake client handles start; never read as memory.
    const CLIENT_BASE: usize = 0x1000_0000;
    /// The local player handle is the client handle plus this.
    const PLAYER_OFFSET: usize = 0x10;
    /// Where the chunk handles of the fakes start; each fake has `CHUNK_SPAN` bytes of them, `CHUNK_STRIDE` apart.
    const CHUNK_BASE: usize = 0x5000_0000;
    const CHUNK_SPAN: usize = 0x10_0000;
    const CHUNK_STRIDE: usize = 8;
    /// Where the block handles of the fakes start; laid out as the chunk handles.
    const BLOCK_BASE: usize = 0x6000_0000;
    /// The held stack handle is the player handle plus this.
    const STACK_OFFSET: usize = 0x10;
    /// The slot stack handles start at the client handle plus this and are this far apart (36 of them fit in the
    /// 0x100 between two fake clients).
    const SLOT_BASE: usize = 0x40;
    const SLOT_STRIDE: usize = 4;

    /// A state that offers only `input.lease`, as the loader's own area book does.
    pub(crate) fn new() -> Self {
        let tag = NEXT_TAG.fetch_add(1, Ordering::Relaxed);
        Self {
            tag,
            bus: EventBus::new(),
            providers: ProviderRegistry::new(),
            gate: Cell::new(false),
            epoch: Cell::new(0),
            logs: RefCell::new(Vec::new()),
            missing_required: RefCell::new(Vec::new()),
            refused: RefCell::new(Vec::new()),
            dirs: RefCell::new(Vec::new()),
            phase: Cell::new(FakePhase::Free),
            areas: RefCell::new(vec![(Area::INPUT_LEASE.id, Area::INPUT_LEASE.version)]),
            client: RefCell::new(None),
            chunks: RefCell::new(Vec::new()),
            blocks: RefCell::new(Vec::new()),
            changed: Cell::new(None),
            last_changed_token: Cell::new(0),
            inventory: Cell::new(FakeInventory::default()),
            lighting: FakeLighting::default(),
            client_writes: RefCell::new(Vec::new()),
            rotation_writes: RefCell::new(Vec::new()),
            builds: RefCell::new(Vec::new()),
            block_rotations: RefCell::new(Vec::new()),
            next_join: Cell::new(1),
            focus_held: Cell::new(false),
            focus_replays: Cell::new(0),
            input: FakeInput::new(tag),
            window: Cell::new(None),
            window_requests: RefCell::new(Vec::new()),
            overlay: FakeOverlay::new(tag),
            ui: FakeUi::default(),
            textures: FakeTextureBook::default(),
            chat: FakeChatBook::default(),
            items: FakeItemBook::default(),
            requests: FakeRequests::default(),
            network: FakeNetwork::default(),
            render: FakeRender::default(),
            block_entities: crate::testing::FakeBlockEntities::default(),
            camera: FakeCameraState::default(),
            camera_free: FakeCameraFree::default(),
            options: FakeOptionsState::default(),
            shapes: FakeShapes::default(),
            time: FakeTime::default(),
        }
    }

    pub(crate) fn ui(&self) -> &FakeUi {
        &self.ui
    }

    pub(crate) fn textures(&self) -> &FakeTextureBook {
        &self.textures
    }

    pub(crate) fn chat(&self) -> &FakeChatBook {
        &self.chat
    }

    pub(crate) fn items(&self) -> &FakeItemBook {
        &self.items
    }

    pub(crate) fn requests(&self) -> &FakeRequests {
        &self.requests
    }

    pub(crate) fn network(&self) -> &FakeNetwork {
        &self.network
    }

    pub(crate) fn render(&self) -> &FakeRender {
        &self.render
    }

    pub(crate) fn block_entities(&self) -> &crate::testing::FakeBlockEntities {
        &self.block_entities
    }

    pub(crate) fn camera(&self) -> &FakeCameraState {
        &self.camera
    }

    pub(crate) fn options(&self) -> &FakeOptionsState {
        &self.options
    }

    pub(crate) fn time(&self) -> &FakeTime {
        &self.time
    }

    pub(crate) fn camera_free(&self) -> &FakeCameraFree {
        &self.camera_free
    }

    pub(crate) fn shapes(&self) -> &FakeShapes {
        &self.shapes
    }

    pub(crate) fn lighting(&self) -> &FakeLighting {
        &self.lighting
    }

    /// The world of the client for a block state call of the area `area`: `NOT_READY` with the gate shut or no world
    /// joined (epoch 0), `NOT_AVAILABLE` without the area, `FAILED` for a client with no `FakeWorld`.
    pub(crate) fn block_world(&self, area: Area) -> Result<crate::testing::FakeWorld, AmmlStatus> {
        if !self.gate.get() || self.epoch.get() == 0 {
            return Err(AmmlStatus::NOT_READY);
        }
        if !self.area_offered(area.id, area.version) {
            return Err(AmmlStatus::NOT_AVAILABLE);
        }
        self.client.borrow().as_ref().and_then(|client| client.world.clone()).ok_or(AmmlStatus::FAILED)
    }

    pub(crate) fn note_block_rotation(&self, block: crate::BlockHandle, quarters: u8) {
        self.block_rotations.borrow_mut().push((block, quarters));
    }

    pub(crate) fn block_rotations(&self) -> Vec<(crate::BlockHandle, u8)> {
        self.block_rotations.borrow().clone()
    }

    /// The handle of the off-hand stack of the local player; never read as memory.
    pub(crate) fn off_stack_handle(&self) -> usize {
        self.stack_handle() + Self::STACK_OFFSET
    }

    /// The handle of the stack in inventory slot `index` (0 to 35); never read as memory.
    pub(crate) fn slot_stack_handle(&self, index: usize) -> usize {
        self.client_handle() + Self::SLOT_BASE + Self::SLOT_STRIDE * index
    }

    /// The inventory slot a stack handle names; `None` for a handle that is no slot handle.
    pub(crate) fn slot_of_handle(&self, handle: usize) -> Option<usize> {
        let offset = handle.checked_sub(self.client_handle() + Self::SLOT_BASE)?;
        (offset % Self::SLOT_STRIDE == 0).then_some(offset / Self::SLOT_STRIDE).filter(|index| *index < 36)
    }

    pub(crate) fn overlay(&self) -> &FakeOverlay {
        &self.overlay
    }

    pub(crate) fn set_window(&self, window: FakeWindow) {
        self.window.set(Some(window));
    }

    /// The loader's checks before a window call in `area`: the area is offered and the window
    /// attached. Gives the window state.
    pub(crate) fn window_check(&self, area: Area) -> Result<FakeWindow, AmmlStatus> {
        if !self.area_offered(area.id, area.version) {
            return Err(AmmlStatus::NOT_AVAILABLE);
        }
        self.window.get().ok_or(AmmlStatus::NOT_READY)
    }

    pub(crate) fn record_window_request(&self, line: String) {
        self.window_requests.borrow_mut().push(line);
    }

    pub(crate) fn window_requests(&self) -> Vec<String> {
        self.window_requests.borrow().clone()
    }

    pub(crate) fn input(&self) -> &FakeInput {
        &self.input
    }

    /// Offers `area` at its version.
    pub(crate) fn offer_area(&self, area: Area) {
        let mut areas = self.areas.borrow_mut();
        match areas.iter_mut().find(|(id, _)| *id == area.id) {
            Some((_, version)) => *version = (*version).max(area.version),
            None => areas.push((area.id, area.version)),
        }
    }

    /// Takes `area` back, as the loader does when a later check finds it cannot serve it.
    pub(crate) fn withdraw_area(&self, area: Area) {
        self.areas.borrow_mut().retain(|(id, _)| *id != area.id);
    }

    pub(crate) fn area_offered(&self, id: &str, version: u32) -> bool {
        self.areas.borrow().iter().any(|(offered, at)| *offered == id && *at >= version)
    }

    /// The loader's checks before an injection into `area`: the gate is open and the area offered.
    pub(crate) fn inject_check(&self, area: Area) -> Result<(), AmmlStatus> {
        if !self.gate.get() {
            return Err(AmmlStatus::NOT_READY);
        }
        if !self.area_offered(area.id, area.version) {
            return Err(AmmlStatus::NOT_AVAILABLE);
        }
        Ok(())
    }

    pub(crate) fn set_client(&self, client: FakeClient) {
        *self.client.borrow_mut() = Some(client);
        self.chunks.borrow_mut().clear();
        self.blocks.borrow_mut().clear();
    }

    pub(crate) fn set_inventory(&self, inventory: FakeInventory) {
        self.inventory.set(inventory);
    }

    pub(crate) fn inventory(&self) -> FakeInventory {
        self.inventory.get()
    }

    /// The handle of the held stack of the local player; never read as memory.
    pub(crate) fn stack_handle(&self) -> usize {
        self.player_handle() + Self::STACK_OFFSET
    }

    /// A handle for the chunk at chunk position (`x`, `z`); never read as memory. The handles of the fake stay valid until
    /// the next `set_client`.
    pub(crate) fn chunk_handle(&self, x: i32, z: i32) -> usize {
        let mut chunks = self.chunks.borrow_mut();
        let index = chunks.iter().position(|at| *at == (x, z)).unwrap_or_else(|| {
            chunks.push((x, z));
            chunks.len() - 1
        });
        Self::CHUNK_BASE + (self.tag as usize) * Self::CHUNK_SPAN + index * Self::CHUNK_STRIDE
    }

    /// The chunk position a handle names; `None` for a handle this fake did not give.
    pub(crate) fn chunk_of_handle(&self, handle: usize) -> Option<(i32, i32)> {
        let offset = handle.checked_sub(Self::CHUNK_BASE + (self.tag as usize) * Self::CHUNK_SPAN)?;
        if offset % Self::CHUNK_STRIDE != 0 {
            return None;
        }
        self.chunks.borrow().get(offset / Self::CHUNK_STRIDE).copied()
    }

    /// A token for the next `BlockChanged` raise: never 0, never the same twice.
    pub(crate) fn next_changed_token(&self) -> u32 {
        let token = self.last_changed_token.get().wrapping_add(1).max(1);
        self.last_changed_token.set(token);
        token
    }

    /// Sets the token and new state of the `BlockChanged` event that runs, and gives the pair that was set before.
    pub(crate) fn replace_changed_state(
        &self,
        running: Option<(u32, crate::BlockHandle)>,
    ) -> Option<(u32, crate::BlockHandle)> {
        self.changed.replace(running)
    }

    /// The token and new state of the `BlockChanged` event that runs.
    pub(crate) fn changed_state(&self) -> Option<(u32, crate::BlockHandle)> {
        self.changed.get()
    }

    /// A handle for the block at `pos`; never read as memory. The handles of the fake stay valid until the next
    /// `set_client`.
    pub(crate) fn block_handle(&self, pos: crate::BlockPos) -> usize {
        let mut blocks = self.blocks.borrow_mut();
        let index = blocks.iter().position(|at| *at == pos).unwrap_or_else(|| {
            blocks.push(pos);
            blocks.len() - 1
        });
        Self::BLOCK_BASE + (self.tag as usize) * Self::CHUNK_SPAN + index * Self::CHUNK_STRIDE
    }

    /// The block position a handle names; `None` for a handle this fake did not give.
    pub(crate) fn block_of_handle(&self, handle: usize) -> Option<crate::BlockPos> {
        let offset = handle.checked_sub(Self::BLOCK_BASE + (self.tag as usize) * Self::CHUNK_SPAN)?;
        if offset % Self::CHUNK_STRIDE != 0 {
            return None;
        }
        self.blocks.borrow().get(offset / Self::CHUNK_STRIDE).copied()
    }

    /// The handle of the fake client; unique per fake, never read as memory.
    pub(crate) fn client_handle(&self) -> usize {
        Self::CLIENT_BASE + (self.tag as usize) * 0x100
    }

    /// The handle of the fake client while one is set, else 0.
    pub(crate) fn current_client(&self) -> usize {
        if self.client.borrow().is_some() { self.client_handle() } else { 0 }
    }

    /// The handle of the local player of the fake client.
    pub(crate) fn player_handle(&self) -> usize {
        self.client_handle() + Self::PLAYER_OFFSET
    }

    /// The loader's checks before a client call in `area`: the gate is open, `handle` is the fake
    /// client and the area is offered. Gives the client state.
    pub(crate) fn client_check(&self, handle: usize, area: Area) -> Result<FakeClient, AmmlStatus> {
        if !self.gate.get() {
            return Err(AmmlStatus::NOT_READY);
        }
        let client =
            self.client.borrow().clone().filter(|_| handle == self.client_handle()).ok_or(AmmlStatus::REFUSED)?;
        if !self.area_offered(area.id, area.version) {
            return Err(AmmlStatus::NOT_AVAILABLE);
        }
        Ok(client)
    }

    /// Changes the client state and keeps `line` as a client write.
    pub(crate) fn write_client(&self, change: impl FnOnce(&mut FakeClient), line: String) {
        if let Some(client) = self.client.borrow_mut().as_mut() {
            change(client);
        }
        self.client_writes.borrow_mut().push(line);
    }

    /// A new join id.
    pub(crate) fn next_join(&self) -> u64 {
        let id = self.next_join.get();
        self.next_join.set(id + 1);
        id
    }

    /// Records a focus loss that a listener cancelled.
    pub(crate) fn hold_focus_loss(&self) {
        self.focus_held.set(true);
    }

    /// A focus gain: the held loss is forgotten.
    pub(crate) fn forget_focus_loss(&self) {
        self.focus_held.set(false);
    }

    /// Takes the held loss for a replay and counts the replay; false when none was held.
    pub(crate) fn replay_focus_loss(&self) -> bool {
        if !self.focus_held.replace(false) {
            return false;
        }
        self.focus_replays.set(self.focus_replays.get() + 1);
        true
    }

    pub(crate) fn focus_replays(&self) -> u32 {
        self.focus_replays.get()
    }

    pub(crate) fn client_writes(&self) -> Vec<String> {
        self.client_writes.borrow().clone()
    }

    /// Records a `set_rotation` and makes the fake client's rotation the written one.
    pub(crate) fn write_rotation(&self, rotation: crate::Rotation, head_yaw: Option<f32>) {
        if let Some(client) = self.client.borrow_mut().as_mut() {
            client.rotation = Some(rotation);
        }
        self.rotation_writes.borrow_mut().push((rotation, head_yaw));
    }

    /// Records a `build_at` the game took.
    pub(crate) fn record_build(&self, pos: crate::BlockPos, face: crate::Face) {
        let face = format!("{face:?}").to_lowercase();
        self.builds.borrow_mut().push(format!("build_at {},{},{} face={face}", pos.x, pos.y, pos.z));
    }

    pub(crate) fn builds(&self) -> Vec<String> {
        self.builds.borrow().clone()
    }

    pub(crate) fn rotation_writes(&self) -> Vec<(crate::Rotation, Option<f32>)> {
        self.rotation_writes.borrow().clone()
    }

    /// Makes `state` the fake loader of this thread. False when the thread already has one.
    pub(crate) fn install(state: &Rc<FakeState>) -> bool {
        CURRENT
            .try_with(|current| {
                let mut current = current.borrow_mut();
                if current.is_some() {
                    return false;
                }
                *current = Some(state.clone());
                true
            })
            .unwrap_or(false)
    }

    /// Clears the fake loader of this thread when it is `state`.
    pub(crate) fn uninstall(state: &Rc<FakeState>) {
        let _ = CURRENT.try_with(|current| {
            let mut current = current.borrow_mut();
            if current.as_ref().is_some_and(|held| Rc::ptr_eq(held, state)) {
                *current = None;
            }
        });
    }

    pub(crate) fn current() -> Option<Rc<FakeState>> {
        CURRENT.try_with(|current| current.borrow().clone()).ok().flatten()
    }

    pub(crate) fn bus(&self) -> &EventBus<FakeListener> {
        &self.bus
    }

    pub(crate) fn gate(&self) -> &Cell<bool> {
        &self.gate
    }

    pub(crate) fn epoch(&self) -> &Cell<u64> {
        &self.epoch
    }

    pub(crate) fn logs(&self) -> Vec<String> {
        self.logs.borrow().clone()
    }

    pub(crate) fn missing_required(&self) -> Vec<String> {
        self.missing_required.borrow().clone()
    }

    pub(crate) fn phase(&self) -> &Cell<FakePhase> {
        &self.phase
    }

    // False outside the mod's load and enable once a driver call set the phase: the loader refuses
    // `listen` and `provide` there.
    fn registering(&self) -> bool {
        self.phase.get() != FakePhase::Running
    }

    pub(crate) fn refuse_listen(&self, event: &'static str) {
        self.refused.borrow_mut().push(event);
    }

    pub(crate) fn set_dir(&self, kind: AmmlDirKind, path: &str) {
        self.dirs.borrow_mut().push((kind, path.into()));
    }

    /// Adds a listener as the loader does: an event that is not offered gives `NOT_AVAILABLE`, and a
    /// required one is noted in `missing_required`.
    pub(crate) fn listen(
        &self,
        event: &str,
        version: u32,
        band: u8,
        required: bool,
        listener: FakeListener,
    ) -> Result<u64, AmmlStatus> {
        if !self.registering() {
            return Err(AmmlStatus::FAILED);
        }
        if self.refused.borrow().contains(&event) {
            return Err(AmmlStatus::REFUSED);
        }
        match self.bus.add(event, version, band, Self::OWNER, listener) {
            Ok(ListenerId(id)) => Ok((u64::from(self.tag) << 32) | (id & Self::ID_MASK)),
            Err(ListenError::UnknownEvent | ListenError::VersionTooNew) => {
                if required {
                    self.missing_required.borrow_mut().push(event.to_string());
                }
                Err(AmmlStatus::NOT_AVAILABLE)
            }
            Err(ListenError::OwnerClosed) => Err(AmmlStatus::FAILED),
        }
    }

    /// Removes a listener. A handle from another fake gives `FAILED`, so its `Listener` leaks its
    /// closure instead of freeing one that fake may still call.
    pub(crate) fn unlisten(&self, handle: u64) -> AmmlStatus {
        if (handle >> 32) as u32 != self.tag {
            return AmmlStatus::FAILED;
        }
        if self.bus.remove(ListenerId(handle & Self::ID_MASK)) { AmmlStatus::OK } else { AmmlStatus::INVALID_ARGUMENT }
    }

    /// Removes every listener of the mod and refuses new ones, as the loader does when it disables
    /// a mod whose listener panicked.
    pub(crate) fn disable_owner(&self) {
        self.bus.remove_owner(Self::OWNER);
    }

    /// Allows listens of the mod again after `disable_owner`, as the loader does before an enable.
    pub(crate) fn reopen_owner(&self) {
        self.bus.reopen_owner(Self::OWNER);
    }

    /// Drops every table the mod provides.
    pub(crate) fn drop_providers(&self) {
        self.providers.drop_owner(Self::OWNER);
    }

    /// Keeps `<level> <message>` followed by ` key=value` per field.
    pub(crate) fn log(&self, level: AmmlLogLevel, message: &str, fields: &[(&str, &str)]) {
        let level = match level.known() {
            Some(KnownLogLevel::Trace) => "trace",
            Some(KnownLogLevel::Debug) => "debug",
            Some(KnownLogLevel::Warn) => "warn",
            Some(KnownLogLevel::Error) => "error",
            Some(KnownLogLevel::Info) | None => "info",
        };
        let mut line = format!("{level} {message}");
        for (key, value) in fields {
            line.push(' ');
            line.push_str(key);
            line.push('=');
            line.push_str(value);
        }
        self.logs.borrow_mut().push(line);
    }

    /// The folder of `kind` set last, as a view that stays valid while the state lives.
    pub(crate) fn dir(&self, kind: AmmlDirKind) -> Option<AmmlStr> {
        self.dirs.borrow().iter().rev().find(|(held, _)| *held == kind).map(|(_, path)| AmmlStr::from_str(path))
    }

    pub(crate) fn provide(&self, name: &str, major: u16, minor: u16, table: usize) -> AmmlStatus {
        if !self.registering() {
            return AmmlStatus::FAILED;
        }
        match self.providers.provide(name, major, minor, Self::OWNER, table) {
            Ok(()) => AmmlStatus::OK,
            Err(_) => AmmlStatus::REFUSED,
        }
    }

    pub(crate) fn require(&self, name: &str, major: u16, min_minor: u16) -> Option<(usize, u16)> {
        self.providers.require(name, major, min_minor)
    }
}
