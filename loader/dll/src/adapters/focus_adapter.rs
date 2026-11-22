use std::ffi::c_void;
use std::sync::OnceLock;

use amml_abi::{AmmlEvent, AmmlEventContext, AmmlPhase, AmmlStatus, AmmlThread, AppFocusGained, AppFocusLost};
use amml_data::{AdapterCatalog, Availability, EntryId, NeedKind};
use amml_runtime::FocusReplayBook;

use crate::{ClientAdapter, ClientDetours, EventHub, FocusGainedDetours, LoaderState, SessionAdapter, ThreadNote};

static ADAPTER: OnceLock<FocusAdapter> = OnceLock::new();

/// The `client.app_focus_lost` and `client.app_focus_gained` events and the `client.focus_replay` area.
///
/// `AppFocusLost` is raised by the loss detour of `ClientDetours` (through `loss_requested`), Before the game's
/// handler; a cancel keeps the game's handler from running and marks a held loss for the mod whose listener cancelled
/// first. `AppFocusGained` is raised by the detour of `FocusGainedDetours`, After the game's handler, and forgets
/// every held loss. `replay` passes a held loss of one mod on to the game's handler once.
pub struct FocusAdapter {
    state: &'static LoaderState,
    book: FocusReplayBook,
    lost_thread: ThreadNote,
    gained_thread: ThreadNote,
}

impl FocusAdapter {
    pub const LOST: &'static str = AppFocusLost::ID;
    pub const GAINED: &'static str = AppFocusGained::ID;
    pub const REPLAY: &'static str = "client.focus_replay";
    const AREA_VERSION: u32 = 1;
    // Loader hooks run before every mod hook on the same function.
    const PRIORITY: i32 = i32::MAX;

    /// Hooks the focus-gain slot when its row resolved, offers each row whose hooks stand, and logs one line per row.
    /// The focus-loss hook is the one `ClientAdapter::install` made, so this runs after it. Works once per process: a
    /// second call hooks nothing and gives the catalog rows.
    pub fn install(state: &'static LoaderState) -> Vec<Availability> {
        let resolved = |name: &str| state.table().get(&EntryId(name.to_string())).is_some();
        let mut rows: Vec<Availability> =
            AdapterCatalog::availability(&resolved).into_iter().filter(Self::is_own_row).collect();
        let adapter = Self {
            state,
            book: FocusReplayBook::new(),
            lost_thread: ThreadNote::new(),
            gained_thread: ThreadNote::new(),
        };
        if ADAPTER.set(adapter).is_err() {
            curia::warn!("the focus adapter is already installed");
            return rows;
        }
        let lost_hooked = ClientDetours::focus_lost_hooked();
        let gained_row = rows.iter().any(|row| row.id == Self::GAINED && row.shape.is_some());
        let gained_hooked = gained_row && FocusGainedDetours::hook().install(state, Self::PRIORITY);
        for row in &mut rows {
            let missing: Vec<&'static str> = match row.id {
                Self::LOST if !lost_hooked => vec![ClientDetours::FOCUS_LOST],
                Self::GAINED if !gained_hooked => vec![FocusGainedDetours::ENTRY],
                Self::REPLAY if !lost_hooked => vec![ClientDetours::FOCUS_LOST],
                Self::REPLAY => [FocusGainedDetours::LISTENER_VTABLE, "MinecraftGame.appPlatformListenerOffset"]
                    .into_iter()
                    .filter(|name| !resolved(name))
                    .collect(),
                _ => Vec::new(),
            };
            if row.shape.is_some() && !missing.is_empty() {
                row.shape = None;
                row.missing = missing;
            }
            Self::offer(state, row);
        }
        rows
    }

    fn is_own_row(row: &Availability) -> bool {
        match row.kind {
            NeedKind::Event => row.id == Self::LOST || row.id == Self::GAINED,
            NeedKind::Area => row.id == Self::REPLAY,
        }
    }

    // Offers `row` when it has a shape and logs the outcome.
    fn offer(state: &LoaderState, row: &Availability) {
        match (row.kind, row.shape) {
            (NeedKind::Area, Some(shape)) => {
                state.areas().offer(row.id, row.version);
                curia::info!("area adapter", { area: row.id, shape: shape });
            }
            (NeedKind::Area, None) => {
                curia::info!("area not available", { area: row.id, missing: row.missing.join(", ") })
            }
            (NeedKind::Event, Some(shape)) => {
                state.events().offer(row.id, row.version);
                curia::info!("event offered", { event: row.id, shape: shape });
            }
            (NeedKind::Event, None) => {
                curia::info!("event not available", { event: row.id, missing: row.missing.join(", ") });
            }
        }
    }

    /// The installed adapter; `None` before `install`.
    pub fn get() -> Option<&'static FocusAdapter> {
        ADAPTER.get()
    }

    /// Runs `body` on the installed adapter; `fallback` before `install`.
    pub(crate) fn with<R>(fallback: R, body: impl FnOnce(&FocusAdapter) -> R) -> R {
        match ADAPTER.get() {
            Some(adapter) => body(adapter),
            None => fallback,
        }
    }

    /// Forgets the held loss of the mod at `owner`, when that mod is removed.
    pub(crate) fn forget_owner(owner: usize) {
        Self::with((), |adapter| {
            adapter.book.forget_owner(owner);
            adapter.note_held();
        });
    }

    // Tells the gain detour whether any mod holds a loss.
    fn note_held(&self) {
        FocusGainedDetours::set(FocusGainedDetours::HELD, self.book.any());
    }

    // The context of a focus event: the game thread with the primary client, else the window thread. The first call
    // on a thread, and each change of thread, logs one line.
    fn context(&self, note: &ThreadNote, name: &'static str) -> AmmlEventContext {
        let hub = self.state.events();
        let game = hub.gate().on_game_thread();
        let thread = if game { AmmlThread::GAME } else { AmmlThread::WINDOW };
        if note.changed(thread) {
            curia::info!("app focus thread", { event: name, game: game });
        }
        let mut context = hub.context_on(thread);
        if game && let Some(client) = SessionAdapter::primary_client() {
            context.client = std::ptr::with_exposed_provenance::<c_void>(client);
        }
        context
    }

    /// Before the game's `onAppFocusLost`: raises `AppFocusLost`. False when a listener cancelled it, so the game's
    /// handler does not run; the mod whose listener cancelled first then holds the loss.
    pub(crate) fn loss_requested(&self) -> bool {
        let hub = self.state.events();
        if !hub.has_listeners(AppFocusLost::ID) {
            return true;
        }
        let context = self.context(&self.lost_thread, Self::LOST);
        let mut event = AppFocusLost { header: EventHub::header::<AppFocusLost>(AmmlPhase::BEFORE) };
        let report = hub.raise(&mut event, &context);
        if event.header.cancelled == 0 {
            return true;
        }
        if let Some(owner) = report.cancelled_by {
            self.book.cancelled(owner);
            self.note_held();
        }
        false
    }

    /// After the game's `onAppFocusGained`: every held loss is forgotten, then `AppFocusGained` is raised while a mod
    /// listens.
    pub(crate) fn gained(&self) {
        if self.book.gained() {
            self.note_held();
        }
        let hub = self.state.events();
        if !hub.has_listeners(AppFocusGained::ID) {
            return;
        }
        let context = self.context(&self.gained_thread, Self::GAINED);
        let mut event = AppFocusGained { header: EventHub::header::<AppFocusGained>(AmmlPhase::AFTER) };
        hub.raise(&mut event, &context);
    }

    /// Passes the loss that a listener of the mod at `owner` cancelled on to the game's handler, once, past the
    /// loader's detour: no `AppFocusLost` is raised. `NOT_READY` while the game gate is closed, `NOT_AVAILABLE`
    /// without area `client.focus_replay`, `REFUSED` off the game thread or for a client that is not the primary
    /// client. `Ok` with nothing run when `owner` holds no loss.
    pub fn replay(&self, owner: usize, client: *const c_void) -> Result<(), AmmlStatus> {
        self.state.events().gate().require_open()?;
        if !self.state.areas().is_available(Self::REPLAY, Self::AREA_VERSION) {
            return Err(AmmlStatus::NOT_AVAILABLE);
        }
        let adapter = ClientAdapter::get().ok_or(AmmlStatus::NOT_AVAILABLE)?;
        let client = adapter.check(client)?;
        if !self.book.replay(owner) {
            return Ok(());
        }
        self.note_held();
        let ran = adapter.listener_of(client).and_then(|listener| adapter.run_focus_lost(listener));
        if ran.is_err() {
            self.book.cancelled(owner);
            self.note_held();
        }
        ran
    }
}
