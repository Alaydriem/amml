use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, MutexGuard, OnceLock};
use std::time::Instant;

use amml_abi::{
    AmmlBlockHandle, AmmlEvent, AmmlInventoryMove, AmmlPhase, AmmlRequestOptions, AmmlStatus, AmmlThread,
    InventoryRequestDone,
};
use amml_data::{AdapterCatalog, Availability, EntryId, NeedKind};
use amml_runtime::{RequestBook, RequestDone, RequestMove, RequestRefusal, RequestStep, RequestTick};

use crate::{
    BlockHandles, ClientAdapter, EventHub, GameRequestPort, InventoryAdapter, LoaderState, PacketDetours, PacketIo,
    RequestCalls, RequestPort, ScreenAdapter, SessionAdapter, ThreadNote, UiFlags,
};

static ADAPTER: OnceLock<RequestAdapter> = OnceLock::new();
static STARTED: AtomicBool = AtomicBool::new(false);
// Whether the refusal of a request off the game thread was logged.
static THREAD_REFUSAL_LOGGED: AtomicBool = AtomicBool::new(false);

/// The `inventory.request` area and its `inventory.request_done` event: moves between the slots of the player inventory
/// as item stack requests the server answers.
///
/// A call only takes the request (`RequestBook::start`); the packets go out at the next client tick, and the end is
/// raised as `InventoryRequestDone` at a client tick after it, never inside the call. One request runs at a time for the
/// whole loader. The book decides under its lock and the adapter acts with no lock held (`RequestPort`), so an answer
/// that comes back inside the game's own send meets no lock; it is recorded and the request is settled at the next tick.
///
/// Every game act is on the game thread (`REFUSED` elsewhere, logged once). The packet detours (`PacketDetours`) hide the
/// server's open of the hidden inventory and its answer to our close, and give the answers of the server to the book.
pub struct RequestAdapter {
    state: &'static LoaderState,
    calls: Option<RequestCalls>,
    io: Option<PacketIo>,
    book: Mutex<RequestBook>,
    epoch: Instant,
    request_thread: ThreadNote,
    tick_thread: ThreadNote,
}

impl RequestAdapter {
    pub const REQUEST: &'static str = "inventory.request";
    pub const REQUEST_DONE: &'static str = InventoryRequestDone::ID;
    const AREA_VERSION: u32 = 1;
    // The version that adds the creative pick.
    const PICK_VERSION: u32 = 2;
    // The hotbar slots a pick can fill.
    const HOTBAR_SLOTS: u8 = 9;
    // Loader hooks run before every mod hook on the same function.
    const PRIORITY: i32 = i32::MAX;

    /// Hooks the three packet handlers when the catalog shape of the area resolved and the inventory layout is the
    /// loader's, offers the area and its event, and logs one line per row. Works once per process: a second call hooks
    /// nothing and gives the catalog rows. The inventory adapter must be installed first.
    pub fn install(state: &'static LoaderState) -> Vec<Availability> {
        let resolved = |name: &str| state.table().get(&EntryId(name.to_string())).is_some();
        let mut rows: Vec<Availability> =
            AdapterCatalog::availability(&resolved).into_iter().filter(Self::is_own_row).collect();
        if STARTED.swap(true, Ordering::AcqRel) {
            curia::warn!("the request adapter is already installed");
            return rows;
        }
        let shaped = rows.iter().any(|row| row.shape.is_some());
        let calls = RequestCalls::new(state).filter(|_| shaped);
        let hooked = calls.is_some()
            && [PacketDetours::open_hook(), PacketDetours::close_hook(), PacketDetours::response_hook()]
                .iter()
                .all(|hook| hook.install(state, Self::PRIORITY));
        // The slots are read through the inventory layout; without it no request can be made.
        let layout = InventoryAdapter::get().is_some_and(|adapter| adapter.layout().is_some());
        let usable = hooked && layout;
        if !Self::install_with(state, calls.filter(|_| hooked)) {
            return rows;
        }
        for row in &mut rows {
            if row.shape.is_some() && !usable {
                row.shape = None;
                row.missing = vec!["a hook, a call or the inventory layout"];
            }
            match (row.shape, row.kind) {
                (Some(shape), NeedKind::Area) => {
                    state.areas().offer(row.id, row.version);
                    curia::info!("area adapter", { area: row.id, shape: shape });
                }
                (Some(shape), NeedKind::Event) => {
                    state.events().offer(row.id, row.version);
                    curia::info!("event offered", { event: row.id, shape: shape });
                }
                (None, NeedKind::Area) => {
                    curia::info!("area not available", { area: row.id, missing: row.missing.join(", ") });
                }
                (None, NeedKind::Event) => {
                    curia::info!("event not available", { event: row.id, missing: row.missing.join(", ") });
                }
            }
        }
        rows
    }

    /// Installs the adapter over `calls` (without them no request takes place). False when an adapter is already
    /// installed. Offers and hooks nothing.
    pub fn install_with(state: &'static LoaderState, calls: Option<RequestCalls>) -> bool {
        let adapter = Self {
            state,
            calls,
            io: calls.map(PacketIo::new),
            book: Mutex::new(RequestBook::new()),
            epoch: Instant::now(),
            request_thread: ThreadNote::new(),
            tick_thread: ThreadNote::new(),
        };
        if ADAPTER.set(adapter).is_err() {
            curia::warn!("the request adapter is already installed");
            return false;
        }
        true
    }

    /// The installed adapter; `None` before `install`.
    pub fn get() -> Option<&'static RequestAdapter> {
        ADAPTER.get()
    }

    /// Runs `body` on the installed adapter; `fallback` before `install`.
    pub(crate) fn with<R>(fallback: R, body: impl FnOnce(&RequestAdapter) -> R) -> R {
        match ADAPTER.get() {
            Some(adapter) => body(adapter),
            None => fallback,
        }
    }

    fn is_own_row(row: &Availability) -> bool {
        row.id == Self::REQUEST || row.id == Self::REQUEST_DONE
    }

    /// The book of the running request, for a test that drives the packet detours.
    pub fn book(&self) -> MutexGuard<'_, RequestBook> {
        self.book.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    /// The seconds since the adapter was installed.
    pub fn now(&self) -> f64 {
        self.epoch.elapsed().as_secs_f64()
    }

    /// Takes a request of the mod `owner` for the local player `player`: gives its id. `NOT_READY` before the game runs,
    /// `NOT_AVAILABLE` without the area or for the pick kind, `REFUSED` off the game thread, for a player that is not
    /// the local player of the primary client, while another request runs and for a hidden open while a screen is open,
    /// `INVALID_ARGUMENT` for a request that names no move, too many moves, a slot that does not exist or a move with
    /// no effect.
    pub fn request_moves(
        &self,
        owner: usize,
        player: *const std::ffi::c_void,
        moves: &[AmmlInventoryMove],
        options: AmmlRequestOptions,
    ) -> Result<u64, AmmlStatus> {
        self.enter(player, Self::AREA_VERSION)?;
        let (Some(calls), Some(io), Some(client)) =
            (self.calls.as_ref(), self.io.as_ref(), SessionAdapter::primary_client())
        else {
            return Err(AmmlStatus::NOT_AVAILABLE);
        };
        let moves = moves.iter().copied().map(Self::decode).collect::<Result<Vec<_>, _>>()?;
        let mut port = GameRequestPort::new(calls, io, client);
        self.request_with(&mut port, owner, &moves, options)
    }

    /// Takes the creative pick of the mod `owner` for the local player `player`: the item of the block `block` into the
    /// empty hotbar slot `slot`, as one request that the next client tick sends. Gives its id. `NOT_READY` before the
    /// game runs, `NOT_AVAILABLE` without the area at version 2, `REFUSED` off the game thread, for a player that is not
    /// the local player, outside the creative game mode, for a block with no creative entry, for a slot above 8 and while
    /// another request runs, `STALE` for a block handle that was taken back.
    pub fn request_pick(
        &self,
        owner: usize,
        player: *const std::ffi::c_void,
        block: AmmlBlockHandle,
        slot: u8,
    ) -> Result<u64, AmmlStatus> {
        self.enter(player, Self::PICK_VERSION)?;
        let (Some(calls), Some(io), Some(client)) =
            (self.calls.as_ref(), self.io.as_ref(), SessionAdapter::primary_client())
        else {
            return Err(AmmlStatus::NOT_AVAILABLE);
        };
        if calls.pick.is_none() {
            return Err(AmmlStatus::NOT_AVAILABLE);
        }
        let mut port = GameRequestPort::new(calls, io, client);
        self.pick_with(&mut port, owner, block, slot)
    }

    /// `request_pick` over a port, after the gate, thread and player checks: no hidden open, and the stack is shown on
    /// the client at once.
    pub fn pick_with(
        &self,
        port: &mut dyn RequestPort,
        owner: usize,
        block: AmmlBlockHandle,
        slot: u8,
    ) -> Result<u64, AmmlStatus> {
        if slot >= Self::HOTBAR_SLOTS {
            return Err(AmmlStatus::REFUSED);
        }
        let address = BlockHandles::address(self.state, block)?;
        if !port.is_creative() {
            return Err(AmmlStatus::REFUSED);
        }
        let entry = port.creative_entry(address).ok_or(AmmlStatus::REFUSED)?;
        let moves = [RequestMove::Pick { slot, creative_net_id: entry.creative_net_id, count: entry.count }];
        self.request_with(port, owner, &moves, AmmlRequestOptions { hidden_open: 0, predict: 1, _reserved: [0; 2] })
    }

    // The checks every call of the area makes before it reads the game: the gate, the area at `version`, the game thread
    // and the player (the local player of the primary client).
    fn enter(&self, player: *const std::ffi::c_void, version: u32) -> Result<(), AmmlStatus> {
        let gate = self.state.events().gate();
        gate.require_open()?;
        if !self.state.areas().is_available(Self::REQUEST, version) {
            return Err(AmmlStatus::NOT_AVAILABLE);
        }
        if !gate.on_game_thread() {
            if !THREAD_REFUSAL_LOGGED.swap(true, Ordering::AcqRel) {
                curia::info!("inventory request refused", { thread: "not game" });
            }
            return Err(AmmlStatus::REFUSED);
        }
        let current = ClientAdapter::get().and_then(ClientAdapter::primary_local_player);
        if current != Some(player.expose_provenance()) {
            return Err(AmmlStatus::REFUSED);
        }
        Ok(())
    }

    /// `request_moves` over a port, after the game checks.
    pub fn request_with(
        &self,
        port: &mut dyn RequestPort,
        owner: usize,
        moves: &[RequestMove],
        options: AmmlRequestOptions,
    ) -> Result<u64, AmmlStatus> {
        let hidden_open = options.hidden_open != 0;
        let screen_open = hidden_open && port.screen_open(true);
        let started = {
            let mut book = self.book();
            let started = book.start(owner, moves, hidden_open, options.predict != 0, screen_open);
            UiFlags::put(UiFlags::REQUEST, !book.is_idle());
            started
        };
        match started {
            Ok(id) => {
                let note = self.request_thread.changed(self.thread());
                if note {
                    curia::info!("request thread", { path: "request_moves", game: self.thread() == AmmlThread::GAME });
                }
                curia::info!("inventory request taken", {
                    id: id,
                    moves: moves.len(),
                    hidden_open: hidden_open,
                    predict: options.predict != 0
                });
                Ok(id)
            }
            Err(RequestRefusal::Busy | RequestRefusal::ScreenOpen) => Err(AmmlStatus::REFUSED),
            Err(RequestRefusal::Empty | RequestRefusal::TooMany | RequestRefusal::BadMove) => {
                Err(AmmlStatus::INVALID_ARGUMENT)
            }
        }
    }

    fn decode(item: AmmlInventoryMove) -> Result<RequestMove, AmmlStatus> {
        match item.kind {
            AmmlInventoryMove::SWAP => Ok(RequestMove::Swap { a: item.a, b: item.b }),
            AmmlInventoryMove::PLACE => Ok(RequestMove::Place { from: item.a, to: item.b, count: item.count }),
            AmmlInventoryMove::DROP => Ok(RequestMove::Drop { from: item.a, count: item.count }),
            AmmlInventoryMove::PICK => Err(AmmlStatus::NOT_AVAILABLE),
            _ => Err(AmmlStatus::INVALID_ARGUMENT),
        }
    }

    /// At `ClientTick` of the primary `client`, on the game thread, before the event: runs the request.
    pub(crate) fn client_tick(&self, client: usize) {
        let (Some(calls), Some(io)) = (self.calls.as_ref(), self.io.as_ref()) else { return };
        if self.book().is_idle() {
            return;
        }
        let mut port = GameRequestPort::new(calls, io, client);
        self.tick_with(&mut port, client);
    }

    /// One tick of the request over `port`: the book decides under its lock, then the acts run with no lock held, then
    /// the ends are raised as events.
    pub fn tick_with(&self, port: &mut dyn RequestPort, client: usize) {
        if self.book().is_idle() {
            UiFlags::clear(UiFlags::REQUEST);
            return;
        }
        if self.tick_thread.changed(self.thread()) {
            curia::info!("request thread", { path: "client_tick", game: self.thread() == AmmlThread::GAME });
        }
        // The game reads come first: no game call runs under the lock of the book.
        let screen_open = port.screen_open(false);
        let mut slots = if self.book().wants_slots() { port.read_slots() } else { None };
        let now = self.now();
        let tick: RequestTick = self.book().tick(now, screen_open, &mut || slots.take());
        let mut done = tick.done;
        done.extend(self.run_steps(port, &tick.steps));
        UiFlags::put(UiFlags::REQUEST, !self.book().is_idle());
        self.raise_done(client, done);
    }

    // Raises `InventoryRequestDone` for each end, on this thread.
    fn raise_done(&self, client: usize, done: Vec<RequestDone>) {
        for done in done {
            curia::info!("inventory request done", {
                id: done.id,
                outcome: done.outcome.0,
                accepted: done.accepted,
                total: done.total
            });
            let mut event = InventoryRequestDone {
                header: EventHub::header::<InventoryRequestDone>(AmmlPhase::AFTER),
                id: done.id,
                outcome: done.outcome.0,
                _reserved: 0,
                accepted: done.accepted,
                total: done.total,
                _reserved2: 0,
            };
            let context = ScreenAdapter::context_of(self.state, Some(client));
            self.state.events().raise(&mut event, &context);
        }
    }

    // Runs the acts in order. A packet the game did not take (`SendInteract`, `SendRequests`) ends the request at once as
    // `NOT_SENT`: the acts after it (the predictions) do not run, and the hidden inventory the server opened is closed.
    // Any other act that fails is logged and the rest go on. Gives the ends that came from a failed send.
    fn run_steps(&self, port: &mut dyn RequestPort, steps: &[RequestStep]) -> Vec<RequestDone> {
        for step in steps {
            if port.run(step) {
                continue;
            }
            curia::warn!("a request step did not run", { step: Self::step_name(step) });
            if !matches!(step, RequestStep::SendInteract | RequestStep::SendRequests(_)) {
                continue;
            }
            let failed = self.book().send_failed(self.now());
            for extra in &failed.steps {
                if !port.run(extra) {
                    curia::warn!("a request step did not run", { step: Self::step_name(extra) });
                }
            }
            return failed.done;
        }
        Vec::new()
    }

    fn step_name(step: &RequestStep) -> &'static str {
        match step {
            RequestStep::SendInteract => "send_interact",
            RequestStep::SendRequests(_) => "send_requests",
            RequestStep::SendClose(_) => "send_close",
            RequestStep::Predict { .. } => "predict",
            RequestStep::Unpredict { .. } => "unpredict",
            RequestStep::PredictPick { .. } => "predict_pick",
            RequestStep::UnpredictPick { .. } => "unpredict_pick",
            RequestStep::AdoptNetId { .. } => "adopt_net_id",
            RequestStep::NetIdMissing { .. } => "net_id_missing",
        }
    }

    fn thread(&self) -> AmmlThread {
        if self.state.events().gate().on_game_thread() { AmmlThread::GAME } else { AmmlThread::RENDER }
    }

    /// The server's `ContainerOpen` is at `packet`: true when it is the open of the hidden inventory, which the game
    /// must not see.
    pub fn container_open(&self, packet: usize) -> bool {
        // SAFETY: `packet` is the live `ContainerOpenPacket` the game passed to its handler.
        let Some((id, kind)) = (unsafe { crate::PacketLayout::container_open(packet) }) else { return false };
        let hidden = self.book().on_container_open(self.now(), id, kind);
        if hidden {
            curia::info!("the server's inventory open was hidden", { container_id: i64::from(id) });
        }
        hidden
    }

    /// The server's `ContainerClose` is at `packet`: true when it is the answer to our close, which the game must not
    /// see.
    pub fn container_close(&self, packet: usize) -> bool {
        // SAFETY: `packet` is the live `ContainerClosePacket` the game passed to its handler.
        let Some(id) = (unsafe { crate::PacketLayout::container_close(packet) }) else { return false };
        self.book().on_container_close(self.now(), id)
    }

    /// The server's `ItemStackResponse` is at `packet`: its answers go to the book. The game always gets the packet.
    pub fn item_stack_response(&self, packet: usize) {
        // SAFETY: `packet` is the live `ItemStackResponsePacket` the game passed to its handler.
        let answers = unsafe { crate::PacketLayout::responses(packet) };
        if !answers.is_empty() {
            // SAFETY: as above.
            let named = unsafe { crate::PacketLayout::hotbar_net_ids(packet) };
            let mut book = self.book();
            book.on_responses(&answers);
            book.on_slot_net_ids(&named);
        }
    }

    /// The mod `owner` is released: its request ends with no event, and while the game runs the hidden inventory it
    /// opened is closed.
    pub fn disable_owner(&self, owner: usize) {
        self.end_request(Some(owner));
    }

    fn end_request(&self, owner: Option<usize>) {
        let gate = self.state.events().gate();
        let alive = gate.is_open() && gate.on_game_thread();
        let steps = {
            let mut book = self.book();
            let steps = book.disable(owner, self.now(), alive);
            UiFlags::put(UiFlags::REQUEST, !book.is_idle());
            steps
        };
        if steps.is_empty() {
            return;
        }
        let (Some(calls), Some(io), Some(client)) =
            (self.calls.as_ref(), self.io.as_ref(), SessionAdapter::primary_client())
        else {
            return;
        };
        let mut port = GameRequestPort::new(calls, io, client);
        let _ = self.run_steps(&mut port, &steps);
    }

    /// A world is left, before the world is taken back: a running request ends `CANCELLED` and its event is raised
    /// now, with nothing sent (the connection is going away).
    pub fn world_left(&self) {
        let done = {
            let mut book = self.book();
            let done = book.world_left();
            UiFlags::clear(UiFlags::REQUEST);
            done
        };
        self.raise_done(SessionAdapter::primary_client().unwrap_or(0), done);
    }
}
