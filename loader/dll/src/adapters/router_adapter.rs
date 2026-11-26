use std::ffi::c_void;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Mutex, OnceLock, PoisonError};

use amml_abi::{AmmlEvent, AmmlEventContext, AmmlPhase, AmmlRouteVia, AmmlStatus, AmmlThread, IdleScreenOpen};
use amml_data::{AdapterCatalog, Availability, EntryId, NeedKind};
use amml_msvc::MsvcString;

use crate::{ClientAdapter, EventHub, LoaderState, RouterDetours, SessionAdapter, SessionHook, ThreadNote};

static ADAPTER: OnceLock<RouterAdapter> = OnceLock::new();

/// The router adapter over `OreUI::Router`: raises `IdleScreenOpen` when the game pushes or replaces its idle route,
/// and keeps the last route the router took (the game's call gave true) for the title read (`client.title_screen`).
/// A route the router was asked for and refused does not count.
///
/// Only a route whose text ends with `IDLE_ROUTE` (the router may add a prefix) raises the event, and only while a
/// mod listens to it; every other route goes to the game untouched. A cancelled event refuses the route: the game
/// gets `false`, as for a route the router does not take.
pub struct RouterAdapter {
    state: &'static LoaderState,
    thread: ThreadNote,
    // the distinct route texts logged so far, at most `MAX_ROUTE_LINES`
    routes: Mutex<Vec<Vec<u8>>>,
    // the text of the last route the router took, and the frame it was taken at
    last: Mutex<(Vec<u8>, u64)>,
    // frames seen (`frame` runs once per frame, After the frame's listeners)
    frames: AtomicU64,
    // `route taken` / `route not taken` lines logged so far, at most `MAX_TAKEN_LINES`
    taken_lines: AtomicUsize,
}

impl RouterAdapter {
    /// The end of the idle screen's route text (`OreUI::EntryPoints::Idle::ROUTE`).
    pub const IDLE_ROUTE: &'static [u8] = b"/idle-route";
    /// The title screen's route (the router's main menu), before any `/` or `?` that follows it.
    pub const TITLE_ROUTE: &'static [u8] = b"/main-menu";
    pub const TITLE_AREA: &'static str = "client.title_screen";
    /// The frames a taken title route must stand, with no later route taken, before the read counts it as shown.
    pub const TITLE_FRAMES: u64 = 30;
    // the `route taken` / `route not taken` lines of one process
    const MAX_TAKEN_LINES: usize = 256;
    const AREA_VERSION: u32 = 1;
    // Loader hooks run before every mod hook on the same function.
    const PRIORITY: i32 = i32::MAX;
    // the distinct routes logged with `route seen`; enough for the menus of one session
    const MAX_ROUTE_LINES: usize = 32;

    /// Hooks both router functions when the `client.idle_screen_open` or `client.title_screen` catalog shape resolved
    /// (once), and offers the event and the area when both hooks went in. Returns those rows; a refused hook names its
    /// entry as missing. Logs `event offered` / `area adapter` or `event not available` / `area not available`. Works
    /// once per process: a second call hooks nothing and gives the catalog rows.
    pub fn install(state: &'static LoaderState) -> Vec<Availability> {
        let resolved = |name: &str| state.table().get(&EntryId(name.to_string())).is_some();
        let mut rows: Vec<Availability> =
            AdapterCatalog::availability(&resolved).into_iter().filter(Self::is_own_row).collect();
        let adapter = Self {
            state,
            thread: ThreadNote::new(),
            routes: Mutex::new(Vec::new()),
            last: Mutex::new((Vec::new(), 0)),
            frames: AtomicU64::new(0),
            taken_lines: AtomicUsize::new(0),
        };
        if ADAPTER.set(adapter).is_err() {
            curia::warn!("the router adapter is already installed");
            return rows;
        }
        let mut refused: Option<Vec<&'static str>> = None;
        for row in &mut rows {
            if row.shape.is_some() {
                let refused = refused.get_or_insert_with(|| {
                    RouterDetours::hooks()
                        .iter()
                        .filter(|hook| !Self::hook(state, hook))
                        .map(|hook| hook.entry)
                        .collect()
                });
                if refused.is_empty() {
                    match row.kind {
                        NeedKind::Event => state.events().offer(row.id, row.version),
                        NeedKind::Area => state.areas().offer(row.id, row.version),
                    }
                } else {
                    row.shape = None;
                    row.missing = refused.clone();
                }
            }
            match (row.kind, row.shape) {
                (NeedKind::Event, Some(shape)) => curia::info!("event offered", { event: row.id, shape: shape }),
                (NeedKind::Area, Some(shape)) => curia::info!("area adapter", { area: row.id, shape: shape }),
                (NeedKind::Event, None) => {
                    curia::info!("event not available", { event: row.id, missing: row.missing.join(", ") })
                }
                (NeedKind::Area, None) => {
                    curia::info!("area not available", { area: row.id, missing: row.missing.join(", ") })
                }
            }
        }
        rows
    }

    /// Runs `body` on the installed adapter; `fallback` before `install`.
    pub(crate) fn with<R>(fallback: R, body: impl FnOnce(&RouterAdapter) -> R) -> R {
        match ADAPTER.get() {
            Some(adapter) => body(adapter),
            None => fallback,
        }
    }

    // Logs `route seen` once for each distinct route text, at most `MAX_ROUTE_LINES` times, so a run shows which
    // routes the two hooked router functions see.
    fn note_route(&self, route: &[u8], via: AmmlRouteVia) {
        let mut routes = self.routes.lock().unwrap_or_else(PoisonError::into_inner);
        if routes.len() >= Self::MAX_ROUTE_LINES || routes.iter().any(|seen| seen == route) {
            return;
        }
        routes.push(route.to_vec());
        curia::info!("route seen", { route: String::from_utf8_lossy(route).into_owned(), via: via.0 });
    }

    fn is_own_row(row: &Availability) -> bool {
        (row.kind == NeedKind::Event && row.id == IdleScreenOpen::ID)
            || (row.kind == NeedKind::Area && row.id == Self::TITLE_AREA)
    }

    /// Whether `client` shows the title screen: the gate is open (`NOT_READY`), the area is offered
    /// (`NOT_AVAILABLE`), the client rules hold (`ClientAdapter::check`), no world is joined, the last route the router
    /// took is `TITLE_ROUTE` (alone or followed by `/` or `?`), and it has stood for `TITLE_FRAMES` frames, so the page
    /// is drawn. A back navigation does not pass through the hooked calls; it can only leave the read false.
    pub fn on_title_screen(&self, client: *const c_void) -> Result<bool, AmmlStatus> {
        self.state.events().gate().require_open()?;
        if !self.state.areas().is_available(Self::TITLE_AREA, Self::AREA_VERSION) {
            return Err(AmmlStatus::NOT_AVAILABLE);
        }
        ClientAdapter::get().ok_or(AmmlStatus::NOT_AVAILABLE)?.check(client)?;
        if self.state.events().gate().epoch().0 != 0 {
            return Ok(false);
        }
        let last = self.last.lock().unwrap_or_else(PoisonError::into_inner);
        let stood = self.frames.load(Ordering::Acquire).saturating_sub(last.1);
        Ok(Self::is_title(&last.0) && stood >= Self::TITLE_FRAMES)
    }

    /// After the listeners of `FrameUpdate` (After): one more frame.
    pub(crate) fn frame(&self) {
        self.frames.fetch_add(1, Ordering::AcqRel);
    }

    /// After the game's router call with `route`: keeps it as the last route when the router took it (`taken`), and
    /// logs `route taken` for a change of route or `route not taken`, at most `MAX_TAKEN_LINES` lines.
    ///
    /// # Safety
    /// `route` is null or points to a live MSVC `std::string` for the whole call.
    pub(crate) unsafe fn note_answer(&self, route: *const c_void, via: AmmlRouteVia, taken: bool) {
        // SAFETY: the caller guarantees `route` is null or a live string; `as_ref` turns null into None.
        let Some(text) = (unsafe { route.cast::<MsvcString>().as_ref() }) else {
            return;
        };
        let text = text.as_bytes();
        if !taken {
            self.log_taken(false, text, via);
            return;
        }
        let mut last = self.last.lock().unwrap_or_else(PoisonError::into_inner);
        if last.0 != text {
            last.0.clear();
            last.0.extend_from_slice(text);
            last.1 = self.frames.load(Ordering::Acquire);
            drop(last);
            self.log_taken(true, text, via);
        }
    }

    fn log_taken(&self, taken: bool, route: &[u8], via: AmmlRouteVia) {
        if self.taken_lines.fetch_add(1, Ordering::Relaxed) >= Self::MAX_TAKEN_LINES {
            return;
        }
        let (route, frame) = (String::from_utf8_lossy(route).into_owned(), self.frames.load(Ordering::Acquire));
        if taken {
            curia::info!("route taken", { route: route, via: via.0, frame: frame });
        } else {
            curia::info!("route not taken", { route: route, via: via.0, frame: frame });
        }
    }

    // `TITLE_ROUTE` alone, or followed by `/` or `?`
    fn is_title(route: &[u8]) -> bool {
        route.strip_prefix(Self::TITLE_ROUTE).is_some_and(|rest| matches!(rest.first(), None | Some(b'/' | b'?')))
    }

    fn hook(state: &'static LoaderState, hook: &SessionHook) -> bool {
        hook.install(state, Self::PRIORITY)
    }

    /// Whether the router call with `route` is refused: true only for the idle route, with a listener, when a
    /// listener cancelled `IdleScreenOpen`. The thread of the call is logged (`idle screen thread`) for the first
    /// call and when it changes, at most `ThreadNote::MAX_LINES` times.
    ///
    /// # Safety
    /// `route` is null or points to a live MSVC `std::string` for the whole call.
    pub(crate) unsafe fn refuses(&self, route: *const c_void, via: AmmlRouteVia) -> bool {
        // SAFETY: the caller guarantees `route` is null or a live string; `as_ref` turns null into None.
        let Some(text) = (unsafe { route.cast::<MsvcString>().as_ref() }) else {
            return false;
        };
        self.note_route(text.as_bytes(), via);
        let hub = self.state.events();
        if !hub.has_listeners(IdleScreenOpen::ID) {
            return false;
        }
        if !text.as_bytes().ends_with(Self::IDLE_ROUTE) {
            return false;
        }
        let game = hub.gate().on_game_thread();
        let thread = if game { AmmlThread::GAME } else { AmmlThread::RENDER };
        if self.thread.changed(thread) {
            curia::info!("idle screen thread", { game: game });
        }
        let client = if game { SessionAdapter::primary_client().unwrap_or(0) } else { 0 };
        let context =
            AmmlEventContext { client: std::ptr::with_exposed_provenance::<c_void>(client), ..hub.context_on(thread) };
        let mut event =
            IdleScreenOpen { header: EventHub::header::<IdleScreenOpen>(AmmlPhase::BEFORE), via, _reserved: 0 };
        let keep = |event: &mut IdleScreenOpen| {
            event.via = via;
            event._reserved = 0;
        };
        hub.raise_keeping(&mut event, &context, &keep);
        event.header.cancelled != 0
    }
}
