use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::atomic::{AtomicU32, Ordering};

use amml_abi::{AmmlDirKind, AmmlLogLevel, AmmlStatus, AmmlStr, KnownLogLevel};
use amml_runtime::{EventBus, ListenError, ListenerId, ProviderRegistry};

use crate::Area;
use crate::testing::{
    FakeChatBook, FakeClient, FakeInput, FakeInventory, FakeItemBook, FakeLighting, FakeListener, FakeNetwork,
    FakeOverlay, FakePhase, FakeRequests, FakeTextureBook, FakeUi, FakeWindow,
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
    client: Cell<Option<FakeClient>>,
    inventory: Cell<FakeInventory>,
    lighting: FakeLighting,
    client_writes: RefCell<Vec<String>>,
    // The id the next `join_server` gives.
    next_join: Cell<u64>,
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
            client: Cell::new(None),
            inventory: Cell::new(FakeInventory::default()),
            lighting: FakeLighting::default(),
            client_writes: RefCell::new(Vec::new()),
            next_join: Cell::new(1),
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

    pub(crate) fn lighting(&self) -> &FakeLighting {
        &self.lighting
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
        self.client.set(Some(client));
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

    /// The handle of the fake client; unique per fake, never read as memory.
    pub(crate) fn client_handle(&self) -> usize {
        Self::CLIENT_BASE + (self.tag as usize) * 0x100
    }

    /// The handle of the fake client while one is set, else 0.
    pub(crate) fn current_client(&self) -> usize {
        if self.client.get().is_some() { self.client_handle() } else { 0 }
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
        let client = self.client.get().filter(|_| handle == self.client_handle()).ok_or(AmmlStatus::REFUSED)?;
        if !self.area_offered(area.id, area.version) {
            return Err(AmmlStatus::NOT_AVAILABLE);
        }
        Ok(client)
    }

    /// Changes the client state and keeps `line` as a client write.
    pub(crate) fn write_client(&self, change: impl FnOnce(&mut FakeClient), line: String) {
        if let Some(mut client) = self.client.get() {
            change(&mut client);
            self.client.set(Some(client));
        }
        self.client_writes.borrow_mut().push(line);
    }

    /// A new join id.
    pub(crate) fn next_join(&self) -> u64 {
        let id = self.next_join.get();
        self.next_join.set(id + 1);
        id
    }

    pub(crate) fn client_writes(&self) -> Vec<String> {
        self.client_writes.borrow().clone()
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
