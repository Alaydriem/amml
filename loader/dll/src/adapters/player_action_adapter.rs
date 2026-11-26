use std::ffi::c_void;
use std::sync::OnceLock;

use amml_abi::{
    AmmlEvent, AmmlEventContext, AmmlEventHeader, AmmlPhase, AmmlThread, BlockBreakContinue, BlockBreakStart,
    BuildBlock, ItemUse, ItemUseOnBlock, PlayerAttack, UsePressed,
};
use amml_data::{AdapterCatalog, Availability, EntryId, NeedKind};

use crate::{
    ClientAdapter, EventHub, LoaderState, PlayerActionDetours, SessionAdapter, SessionHook, ThreadNote, UseBuildDetours,
};

static ADAPTER: OnceLock<PlayerActionAdapter> = OnceLock::new();

/// The player-action events (`player.block_break_start`, `player.block_break_continue`, `player.item_use`,
/// `player.attack`, `player.item_use_on_block`, `player.use_pressed`, `player.build_block`), raised from the loader's
/// hooks on the `GameMode` slots, `Item::useOn` and `ClientInputCallbacks::handleBuildAction`. Every hook on a
/// `GameMode` slot goes in through this adapter, so one slot target is patched once.
///
/// The hooks sit on the functions in the base `GameMode` table, so the survival wrappers that jump to them are
/// covered; the survival attack is its own leaf, so `player.attack` hooks the slot of both tables (once when both
/// hold the same function). An event is raised only when a mod listens to it, the game gate is open and the call
/// runs on the game thread; any other call goes to the original with no game read. `is_local_player` is 1 when the
/// action's player (`GameMode::mPlayer`, or the actor of `useOn`) is the local player of the primary client. The
/// context carries the primary client.
///
/// `player.build_block` is the one event raised off the game thread too: on the thread that builds (`GAME` with the
/// primary client in the context, or `SERVER` with no client), with the origin of the innermost mod build of that
/// thread (`UseBuildDetours::reentry`).
pub struct PlayerActionAdapter {
    state: &'static LoaderState,
    // The offset of `Player& GameMode::mPlayer`.
    player_offset: Option<usize>,
    build_thread: ThreadNote,
}

impl PlayerActionAdapter {
    // Loader hooks run before every mod hook on the same function.
    const PRIORITY: i32 = i32::MAX;
    /// The `player.item_use` shape of a build whose `useItem` returns a bool.
    pub const BOOL_RESULT: &'static str = "bool_result";
    /// The `player.item_use` shape of a build whose `useItem` returns an `InteractionResult`.
    pub const INTERACTION_RESULT: &'static str = "interaction_result";

    /// Hooks the functions of each event whose catalog shape resolved, offers each event whose hooks went in, and
    /// logs `event offered` or `event not available` per event. Returns the rows. Works once per process.
    pub fn install(state: &'static LoaderState) -> Vec<Availability> {
        let resolved = |name: &str| state.table().get(&EntryId(name.to_string())).is_some();
        let mut rows: Vec<Availability> =
            AdapterCatalog::availability(&resolved).into_iter().filter(Self::is_own_row).collect();
        let player_offset = state.resolve("GameMode.mPlayer").ok().and_then(|value| usize::try_from(value).ok());
        if ADAPTER.set(Self { state, player_offset, build_thread: ThreadNote::new() }).is_err() {
            curia::warn!("the player action adapter is already installed");
            return rows;
        }
        for row in &mut rows {
            if let Some(shape) = row.shape {
                let hooks = Self::hooks(row.id, shape);
                let refused: Vec<&'static str> =
                    Self::install_hooks(state, &hooks).into_iter().map(|hook| hook.entry).collect();
                if refused.is_empty() && !hooks.is_empty() {
                    state.events().offer(row.id, row.version);
                } else {
                    row.shape = None;
                    row.missing = refused;
                }
            }
            match row.shape {
                Some(shape) => curia::info!("event offered", { event: row.id, shape: shape }),
                None => curia::info!("event not available", { event: row.id, missing: row.missing.join(", ") }),
            }
        }
        rows
    }

    /// Runs `body` on the installed adapter; `fallback` before `install`.
    pub(crate) fn with<R>(fallback: R, body: impl FnOnce(&PlayerActionAdapter) -> R) -> R {
        match ADAPTER.get() {
            Some(adapter) => body(adapter),
            None => fallback,
        }
    }

    /// A header for a player-action event: phase Before, raised by the game.
    pub fn header<E: AmmlEvent>() -> AmmlEventHeader {
        EventHub::header::<E>(AmmlPhase::BEFORE)
    }

    fn is_own_row(row: &Availability) -> bool {
        row.kind == NeedKind::Event
            && [
                BlockBreakStart::ID,
                BlockBreakContinue::ID,
                ItemUse::ID,
                PlayerAttack::ID,
                ItemUseOnBlock::ID,
                UsePressed::ID,
                BuildBlock::ID,
            ]
            .contains(&row.id)
    }

    // The hooks of `event` in its shape `shape`.
    fn hooks(event: &str, shape: &str) -> Vec<SessionHook> {
        match (event, shape) {
            (BlockBreakStart::ID, _) => vec![PlayerActionDetours::start()],
            (BlockBreakContinue::ID, _) => vec![PlayerActionDetours::continue_destroy()],
            (ItemUse::ID, Self::BOOL_RESULT) => vec![PlayerActionDetours::use_item_bool()],
            (ItemUse::ID, Self::INTERACTION_RESULT) => vec![PlayerActionDetours::use_item_interaction()],
            (PlayerAttack::ID, _) => vec![PlayerActionDetours::attack(), PlayerActionDetours::survival_attack()],
            (ItemUseOnBlock::ID, _) => vec![PlayerActionDetours::use_on()],
            (UsePressed::ID, _) => vec![UseBuildDetours::use_pressed()],
            (BuildBlock::ID, _) => vec![UseBuildDetours::build_block()],
            _ => Vec::new(),
        }
    }

    // Installs `hooks`, skipping a hook whose target an earlier hook of the list already has; gives the refused
    // hooks.
    fn install_hooks<'h>(state: &'static LoaderState, hooks: &'h [SessionHook]) -> Vec<&'h SessionHook> {
        let mut targets: Vec<*const c_void> = Vec::new();
        let mut refused = Vec::new();
        for hook in hooks {
            let Some(target) = hook.target(state) else {
                refused.push(hook);
                continue;
            };
            if targets.contains(&target) {
                curia::info!("player action hook shared", { entry: hook.entry, vtable: hook.vtable.unwrap_or("") });
                continue;
            }
            if hook.install(state, Self::PRIORITY) {
                targets.push(target);
            } else {
                refused.push(hook);
            }
        }
        refused
    }

    /// Before the game's `handleBuildAction`: raises `UsePressed` when a mod listens, the gate is open and the call runs
    /// on the game thread. True when a listener cancelled it.
    pub(crate) fn use_pressed(&self) -> bool {
        let hub = self.state.events();
        if !hub.has_listeners(UsePressed::ID) || !hub.gate().is_open() || !hub.gate().on_game_thread() {
            return false;
        }
        let mut event = UsePressed { header: Self::header::<UsePressed>() };
        hub.raise(&mut event, &self.client_context());
        event.header.cancelled != 0
    }

    /// Before the base body of `GameMode::buildBlock` on the GameMode `this`, with the fields of `event` from the game's
    /// arguments; `flags` is the flags word of `UseBuildDetours`. On the game thread the player is read from
    /// `GameMode::mPlayer` and compared with the local player of the primary client; with `SIM_TICK` the isSimTick of a
    /// local build is kept. With `BUILD_LISTENER` and the gate open, `BuildBlock` is raised on this thread: the game
    /// thread with the primary client, any other thread as `SERVER` with no client and `is_local_player` 0. True when a
    /// listener cancelled it.
    pub(crate) fn build_block(&self, this: *mut c_void, mut event: BuildBlock, flags: u32) -> bool {
        let hub = self.state.events();
        let game = hub.gate().on_game_thread();
        if game {
            let owner = self.player_of(this).unwrap_or(0);
            let local = ClientAdapter::get().and_then(ClientAdapter::primary_local_player);
            let is_local = owner != 0 && local == Some(owner);
            if is_local && flags & UseBuildDetours::SIM_TICK != 0 {
                UseBuildDetours::note_sim_tick(hub.gate().epoch().0, event.sim_tick);
            }
            event.is_local_player = u8::from(is_local);
        }
        if flags & UseBuildDetours::BUILD_LISTENER == 0 || !hub.gate().is_open() || !hub.has_listeners(BuildBlock::ID) {
            return false;
        }
        let thread = if game { AmmlThread::GAME } else { AmmlThread::SERVER };
        if self.build_thread.changed(thread) {
            curia::info!("build block thread", { thread: thread.0, game: game });
        }
        let context = if game { self.client_context() } else { hub.context_on(AmmlThread::SERVER) };
        match UseBuildDetours::reentry().actor() {
            Some(actor) => hub.raise_acted(&mut event, &context, actor),
            None => hub.raise(&mut event, &context),
        };
        event.header.cancelled != 0
    }

    // The context of an event on the game thread: the primary client.
    fn client_context(&self) -> AmmlEventContext {
        let hub = self.state.events();
        let client = SessionAdapter::primary_client().unwrap_or(0);
        AmmlEventContext { client: std::ptr::with_exposed_provenance(client), ..hub.loader_context() }
    }

    // `GameMode::mPlayer` of the GameMode `this`; `None` for a null game mode or an unknown offset.
    fn player_of(&self, this: *mut c_void) -> Option<usize> {
        let offset = self.player_offset?;
        let address = this.expose_provenance().checked_add(offset).filter(|_| !this.is_null())?;
        // SAFETY: `this` is the GameMode the game passed to one of its own virtuals, a live object for the call, and
        // `offset` is the derived offset of its player pointer.
        Some(unsafe { std::ptr::with_exposed_provenance::<usize>(address).read_unaligned() })
    }

    /// One action of the GameMode `this`: the player is read from `GameMode::mPlayer` only after the listener,
    /// gate and thread tests. True when a listener cancelled the event built by `build`.
    pub(crate) fn game_mode_action<E: AmmlEvent + Clone>(
        &self,
        this: *mut c_void,
        build: impl FnOnce(u8) -> E,
    ) -> bool {
        if self.player_offset.is_none() {
            return false;
        }
        self.raise_with(|| self.player_of(this), build)
    }

    /// One action of the actor `actor` (the `Actor&` of `useOn`). True when a listener cancelled it.
    pub(crate) fn actor_action<E: AmmlEvent + Clone>(&self, actor: usize, build: impl FnOnce(u8) -> E) -> bool {
        self.raise_with(|| Some(actor), build)
    }

    // Raises the event `build` gives for the player `owner` gives, when a mod listens, the gate is open and the call
    // runs on the game thread; true when a listener cancelled it.
    fn raise_with<E: AmmlEvent + Clone>(
        &self,
        owner: impl FnOnce() -> Option<usize>,
        build: impl FnOnce(u8) -> E,
    ) -> bool {
        let hub = self.state.events();
        if !hub.has_listeners(E::ID) || !hub.gate().is_open() || !hub.gate().on_game_thread() {
            return false;
        }
        let owner = owner().unwrap_or(0);
        let local = ClientAdapter::get().and_then(ClientAdapter::primary_local_player);
        let is_local = owner != 0 && local == Some(owner);
        let mut event = build(u8::from(is_local));
        hub.raise(&mut event, &self.client_context());
        Self::cancelled(&event)
    }

    fn cancelled<E: AmmlEvent>(event: &E) -> bool {
        // SAFETY: every player-action event starts with an `AmmlEventHeader`.
        let header = unsafe { &*(event as *const E).cast::<AmmlEventHeader>() };
        header.cancelled != 0
    }
}
