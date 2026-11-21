use std::collections::HashMap;
use std::ffi::c_void;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::Instant;

use amml_abi::{
    AmmlEvent, AmmlEventContext, AmmlPhase, AmmlReleaseReason, ClientTick, FrameUpdate, GameTick, JoinStarted,
    WorldJoined, WorldLeft,
};
use amml_data::{AdapterCatalog, Availability, EntryId, NeedKind};

use crate::{
    ClientAdapter, EventHub, InlineIconAdapter, InventoryAdapter, ItemDrawAdapter, LevelRendererAdapter,
    LightingAdapter, LoaderState, ModLoader, RequestAdapter, ScreenAdapter, SessionActions, SessionDetours,
    SessionHook, TestWorld, TextureAdapter, TextureReloadAdapter,
};

static ADAPTER: OnceLock<SessionAdapter> = OnceLock::new();

/// Raises the session events (`frame.update`, `client.tick`, `client.game_tick`, `world.join_started`,
/// `world.joined`, `world.left`) from loader-owned detours on the game's session functions.
///
/// The primary client is the first `ClientInstance` whose update runs, at startup or after a `WorldLeft`;
/// `client.tick` and `world.joined` come only from it. A world is joined at the first primary client
/// update after `JoinStarted` (or after startup) whose local player read gives non-null; each join takes
/// a new epoch from the game gate. A level exit before that keeps the join waiting. `WorldLeft` follows
/// only a `WorldJoined`, and is offered only when `world.joined` is; after its listeners, the resources of
/// that epoch are taken back and the gate epoch is 0 until the next join. When the `onStartJoinGame` hook
/// is not installed, a leave waits for the next join on its own.
pub struct SessionAdapter {
    state: &'static LoaderState,
    // The slot of `IClientInstance::getLocalPlayer`; `None` when `world.joined` is not available.
    local_player_slot: Option<usize>,
    // The offset of the GameCallbacks subobject in a ClientInstance; `None` when `client.game_tick` is not available.
    callbacks_offset: Option<usize>,
    primary: AtomicUsize,
    game: AtomicUsize,
    awaiting: AtomicBool,
    joined: AtomicBool,
    rearm_on_leave: AtomicBool,
}

impl SessionAdapter {
    const LOCAL_PLAYER: &'static str = "ClientInstance.getLocalPlayer";
    const CALLBACKS_OFFSET: &'static str = "ClientInstance.gameCallbacksOffset";
    const LOADER_SHAPE: &'static str = "loader";
    // Loader hooks run before every mod hook on the same function.
    const PRIORITY: i32 = i32::MAX;

    /// Hooks each function whose catalog shape resolved and offers its events. Returns one row per catalog
    /// event; an event whose hook was refused is not offered and names its entry as missing. Logs
    /// `event offered` or `event not available` for each event. Works once per process: a second call
    /// hooks nothing and gives the catalog rows.
    pub fn install(state: &'static LoaderState) -> Vec<Availability> {
        let mut rows = AdapterCatalog::availability(&|name| state.table().get(&EntryId(name.to_string())).is_some());
        let joined_row = rows.iter().find(|row| row.id == WorldJoined::ID);
        let local_player_slot = joined_row
            .filter(|row| row.shape.is_some())
            .and_then(|_| state.resolve(Self::LOCAL_PLAYER).ok())
            .and_then(|slot| usize::try_from(slot).ok());
        let callbacks_offset = rows
            .iter()
            .find(|row| row.id == GameTick::ID)
            .filter(|row| row.shape.is_some())
            .and_then(|_| state.resolve(Self::CALLBACKS_OFFSET).ok())
            .and_then(|offset| usize::try_from(offset).ok());
        let adapter = Self {
            state,
            local_player_slot,
            callbacks_offset,
            primary: AtomicUsize::new(0),
            game: AtomicUsize::new(0),
            awaiting: AtomicBool::new(true),
            joined: AtomicBool::new(false),
            rearm_on_leave: AtomicBool::new(true),
        };
        if ADAPTER.set(adapter).is_err() {
            curia::warn!("the session adapter is already installed");
            return rows;
        }
        let mut hooked: HashMap<&'static str, bool> = HashMap::new();
        // `world.left` comes after `world.joined`, whose install result it takes.
        let mut order: Vec<usize> = (0..rows.len()).filter(|index| rows[*index].id != WorldLeft::ID).collect();
        order.extend((0..rows.len()).filter(|index| rows[*index].id == WorldLeft::ID));
        let mut joined_missing: Option<Vec<&'static str>> = None;
        for index in order {
            let row = &mut rows[index];
            if !Self::is_session_row(row) {
                continue;
            }
            if row.id == WorldLeft::ID
                && row.shape.is_some()
                && let Some(missing) = joined_missing.as_ref()
            {
                row.shape = None;
                row.missing.clone_from(missing);
            }
            if row.shape.is_some_and(|shape| shape != Self::LOADER_SHAPE) {
                let entry = Self::hook_entry(row.id);
                let installed =
                    entry.is_some_and(|entry| *hooked.entry(entry).or_insert_with(|| Self::hook(state, entry)));
                if installed {
                    state.events().offer(row.id, row.version);
                } else {
                    row.shape = None;
                    row.missing = entry.into_iter().collect();
                }
            }
            if row.id == WorldJoined::ID && row.shape.is_none() {
                joined_missing = Some(row.missing.clone());
            }
            match row.shape {
                Some(shape) => curia::info!("event offered", { event: row.id, shape: shape }),
                None => curia::info!("event not available", { event: row.id, missing: row.missing.join(", ") }),
            }
        }
        let join_hooked = hooked.get(SessionDetours::START_JOIN).copied().unwrap_or(false);
        if let Some(adapter) = ADAPTER.get() {
            adapter.rearm_on_leave.store(!join_hooked, Ordering::Release);
        }
        rows
    }

    /// Runs `body` on the installed adapter; nothing before `install`.
    pub(crate) fn with(body: impl FnOnce(&SessionAdapter)) {
        if let Some(adapter) = ADAPTER.get() {
            body(adapter);
        }
    }

    // Only events this adapter hooks or the loader raises are its to offer and log; other events and
    // every area belong to the adapter that installs them.
    fn is_session_row(row: &Availability) -> bool {
        row.kind == NeedKind::Event && (Self::hook_entry(row.id).is_some() || row.shape == Some(Self::LOADER_SHAPE))
    }

    // The function whose detour raises `event`.
    fn hook_entry(event: &str) -> Option<&'static str> {
        match event {
            FrameUpdate::ID => Some(SessionDetours::GAME_UPDATE),
            ClientTick::ID | WorldJoined::ID => Some(SessionDetours::CLIENT_UPDATE),
            GameTick::ID => Some(SessionDetours::ON_TICK),
            JoinStarted::ID => Some(SessionDetours::START_JOIN),
            WorldLeft::ID => Some(SessionDetours::EXIT_LEVEL),
            _ => None,
        }
    }

    // Installs the detour of `entry`; false, with a log line, when the target is unknown or the hook is refused.
    fn hook(state: &'static LoaderState, entry: &'static str) -> bool {
        SessionDetours::hooks()
            .into_iter()
            .find(|hook| hook.entry == entry)
            .is_some_and(|hook: SessionHook| hook.install(state, Self::PRIORITY))
    }

    /// The primary `ClientInstance`; `None` until its first update and after a world is left.
    pub fn primary_client() -> Option<usize> {
        ADAPTER.get().map(|adapter| adapter.primary.load(Ordering::Acquire)).filter(|client| *client != 0)
    }

    /// The `MinecraftGame` of the last frame; `None` before the first one.
    pub fn minecraft_game() -> Option<usize> {
        ADAPTER.get().map(|adapter| adapter.game.load(Ordering::Acquire)).filter(|game| *game != 0)
    }

    fn hub(&self) -> &EventHub {
        self.state.events()
    }

    // The loader context with `client` as the client.
    fn context(&self, client: usize) -> AmmlEventContext {
        AmmlEventContext { client: std::ptr::with_exposed_provenance::<c_void>(client), ..self.hub().loader_context() }
    }

    /// `FrameUpdate` around `MinecraftGame.update` of `game`. Before the frame, the gate opens (the first
    /// time), this thread becomes the game thread, the mods queued after a listener panic are disabled, the
    /// screenshot requests that waited too long fail, the queued lease and screenshot notices are raised and the queued injected input is applied, all before the
    /// `FrameUpdate` listeners run. A shutdown the window thread queued then runs on the primary client,
    /// and that frame raises no `FrameUpdate` Before. After the frame's `FrameUpdate` listeners, a
    /// `DimensionChanging` kept from another thread is raised and the test world runs.
    pub(crate) fn frame(&self, phase: AmmlPhase, game: *mut c_void) {
        if phase == AmmlPhase::BEFORE {
            self.hub().gate().open();
            self.hub().gate().note_game_thread();
            InventoryAdapter::frame_began();
            self.game.store(game.expose_provenance(), Ordering::Release);
            self.hub().deferred().drain(&ModLoader::new(self.state));
            self.state.capture().expire_shots(Instant::now());
            self.state.owner_events().drain(self.hub());
            self.state.input().apply_frame(self.hub());
            if self.state.shutdown().is_queued() {
                let client = self.primary.load(Ordering::Acquire);
                ModLoader::new(self.state).shutdown_live(client);
                return;
            }
        }
        let mut event = FrameUpdate { header: EventHub::header::<FrameUpdate>(phase) };
        self.hub().raise(&mut event, &self.hub().loader_context());
        if phase == AmmlPhase::AFTER {
            LevelRendererAdapter::with((), LevelRendererAdapter::flush_pending);
            TextureReloadAdapter::with((), TextureReloadAdapter::flush_pending);
            TestWorld::with(TestWorld::frame);
        }
    }

    /// After `ClientInstance.update` of `this`: `WorldJoined` when a join waits and the local player is
    /// there, then the client flags queued for restore are written back, then `ClientTick`; all for the
    /// primary client only.
    pub(crate) fn client_updated(&self, this: *mut c_void) {
        let client = this.expose_provenance();
        if client == 0 {
            return;
        }
        match self.primary.compare_exchange(0, client, Ordering::AcqRel, Ordering::Acquire) {
            Ok(_) => {}
            Err(primary) if primary == client => {}
            Err(_) => return,
        }
        if self.awaiting.load(Ordering::Acquire) && !self.joined.load(Ordering::Acquire) && self.has_local_player(this)
        {
            self.awaiting.store(false, Ordering::Release);
            self.joined.store(true, Ordering::Release);
            let epoch = self.hub().gate().next_epoch();
            let mut joined = WorldJoined { header: EventHub::header::<WorldJoined>(AmmlPhase::AFTER), epoch: epoch.0 };
            self.hub().raise(&mut joined, &self.context(client));
            SessionActions::with(SessionActions::world_joined);
            TextureAdapter::with((), |adapter| adapter.rebind("world_joined"));
        }
        ClientAdapter::with((), |adapter| adapter.apply_restores(client));
        ScreenAdapter::with((), |adapter| adapter.client_tick(client));
        RequestAdapter::with((), |adapter| adapter.client_tick(client));
        let mut tick = ClientTick { header: EventHub::header::<ClientTick>(AmmlPhase::AFTER) };
        self.hub().raise(&mut tick, &self.context(client));
    }

    // Whether `getLocalPlayer` on `client` gives non-null; false when its slot is unknown, or the client,
    // its vtable or the slot is null.
    fn has_local_player(&self, client: *mut c_void) -> bool {
        type LocalPlayerFn = unsafe extern "C" fn(*mut c_void) -> *mut c_void;
        let Some(slot) = self.local_player_slot else {
            return false;
        };
        if client.is_null() {
            return false;
        }
        // SAFETY: `client` is the live, non-null ClientInstance the game passed to its own update; its first
        // word is its IClientInstance vtable pointer.
        let vtable = unsafe { client.cast::<*const usize>().read() };
        if vtable.is_null() {
            return false;
        }
        // SAFETY: `vtable` is the non-null IClientInstance vtable of `client`, which holds `getLocalPlayer`
        // at `slot`.
        let function = unsafe { vtable.add(slot).read() };
        if function == 0 {
            return false;
        }
        // SAFETY: the slot holds `LocalPlayer* IClientInstance::getLocalPlayer() const`, called on its own client.
        let local_player = unsafe { std::mem::transmute::<usize, LocalPlayerFn>(function) };
        // SAFETY: as above.
        !unsafe { local_player(client) }.is_null()
    }

    /// `GameTick` after `ClientInstance.onTick` on the GameCallbacks subobject `callbacks`, only when its client
    /// (`callbacks` minus the derived `ClientInstance.gameCallbacksOffset`) is the primary client, with that client
    /// in the context. No tick is raised before the primary client is known; a negative tick is 0.
    pub(crate) fn ticked(&self, callbacks: *mut c_void, tick: i32) {
        let Some(offset) = self.callbacks_offset else {
            return;
        };
        let Some(client) = callbacks.expose_provenance().checked_sub(offset) else {
            return;
        };
        let primary = self.primary.load(Ordering::Acquire);
        if primary == 0 || primary != client {
            return;
        }
        let tick = u64::try_from(tick).unwrap_or(0);
        let mut event = GameTick { header: EventHub::header::<GameTick>(AmmlPhase::AFTER), tick };
        self.hub().raise(&mut event, &self.context(client));
    }

    /// `JoinStarted` after `ClientInstance.onStartJoinGame`; the next primary client update with a local
    /// player joins the world.
    pub(crate) fn join_started(&self) {
        self.awaiting.store(true, Ordering::Release);
        let mut event = JoinStarted { header: EventHub::header::<JoinStarted>(AmmlPhase::AFTER) };
        self.hub().raise(&mut event, &self.hub().loader_context());
    }

    /// Before `ClientExitLevel.hook`, and first in a live shutdown: a `DimensionChanging` held for the next frame is dropped, `WorldLeft` for a
    /// joined world, then the gate epoch is 0, its resources
    /// are taken back, every injected input is released and the primary client is forgotten. An exit with no joined world changes nothing, so
    /// a join that waits for its local player stays waiting.
    pub(crate) fn leaving(&self) {
        if self.rearm_on_leave.load(Ordering::Acquire) {
            self.awaiting.store(true, Ordering::Release);
        }
        if !self.joined.swap(false, Ordering::AcqRel) {
            return;
        }
        LevelRendererAdapter::with((), LevelRendererAdapter::drop_pending);
        ClientAdapter::with((), ClientAdapter::leave_started);
        let client = self.primary.load(Ordering::Acquire);
        let epoch = self.hub().gate().epoch();
        let mut event = WorldLeft { header: EventHub::header::<WorldLeft>(AmmlPhase::BEFORE), epoch: epoch.0 };
        self.hub().raise(&mut event, &self.context(client));
        LightingAdapter::with((), LightingAdapter::world_left);
        TextureAdapter::with((), TextureAdapter::world_left);
        InlineIconAdapter::with((), InlineIconAdapter::world_left);
        ItemDrawAdapter::with((), ItemDrawAdapter::world_left);
        RequestAdapter::with((), RequestAdapter::world_left);
        // The gate epoch is 0 before the store takes the world back, so a new area call gives
        // `NotReady`; one that read the old epoch earlier gets `Stale` from the store.
        self.hub().gate().end_world();
        let freed = self.state.resources().take_epoch(epoch);
        if !freed.is_empty() {
            curia::info!("resources of a left world taken back", { epoch: epoch.0, count: freed.len() });
        }
        // The ups and the `Released(WORLD_LEFT)` notices reach the game and the mods at the next frame.
        self.state.input().release_everything(AmmlReleaseReason::WORLD_LEFT);
        self.primary.store(0, Ordering::Release);
    }
}
