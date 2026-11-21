use std::ffi::c_void;
use std::sync::{Mutex, OnceLock, PoisonError};

use amml_abi::{AmmlEvent, AmmlEventContext, AmmlPhase, AmmlRouteVia, AmmlThread, IdleScreenOpen};
use amml_data::{AdapterCatalog, Availability, EntryId, NeedKind};
use amml_msvc::MsvcString;

use crate::{EventHub, LoaderState, RouterDetours, SessionAdapter, SessionHook, ThreadNote};

static ADAPTER: OnceLock<RouterAdapter> = OnceLock::new();

/// The router adapter over `OreUI::Router`: raises `IdleScreenOpen` when the game pushes or replaces its idle route.
///
/// Only a route whose text ends with `IDLE_ROUTE` (the router may add a prefix) raises the event, and only while a
/// mod listens to it; every other route goes to the game untouched. A cancelled event refuses the route: the game
/// gets `false`, as for a route the router does not take.
pub struct RouterAdapter {
    state: &'static LoaderState,
    thread: ThreadNote,
    // the distinct route texts logged so far, at most `MAX_ROUTE_LINES`
    routes: Mutex<Vec<Vec<u8>>>,
}

impl RouterAdapter {
    /// The end of the idle screen's route text (`OreUI::EntryPoints::Idle::ROUTE`).
    pub const IDLE_ROUTE: &'static [u8] = b"/idle-route";
    // Loader hooks run before every mod hook on the same function.
    const PRIORITY: i32 = i32::MAX;
    // the distinct routes logged with `route seen`; enough for the menus of one session
    const MAX_ROUTE_LINES: usize = 32;

    /// Hooks both router functions when the `client.idle_screen_open` catalog shape resolved, and offers the event
    /// when both hooks went in. Returns that row; a refused hook names its entry as missing. Logs `event offered` or
    /// `event not available`. Works once per process: a second call hooks nothing and gives the catalog row.
    pub fn install(state: &'static LoaderState) -> Vec<Availability> {
        let resolved = |name: &str| state.table().get(&EntryId(name.to_string())).is_some();
        let mut rows: Vec<Availability> =
            AdapterCatalog::availability(&resolved).into_iter().filter(Self::is_own_row).collect();
        if ADAPTER.set(Self { state, thread: ThreadNote::new(), routes: Mutex::new(Vec::new()) }).is_err() {
            curia::warn!("the router adapter is already installed");
            return rows;
        }
        for row in &mut rows {
            if row.shape.is_some() {
                let refused: Vec<&'static str> = RouterDetours::hooks()
                    .iter()
                    .filter(|hook| !Self::hook(state, hook))
                    .map(|hook| hook.entry)
                    .collect();
                if refused.is_empty() {
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
        row.kind == NeedKind::Event && row.id == IdleScreenOpen::ID
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
        let hub = self.state.events();
        if !hub.has_listeners(IdleScreenOpen::ID) {
            return false;
        }
        // SAFETY: the caller guarantees `route` is null or a live string; `as_ref` turns null into None.
        let Some(text) = (unsafe { route.cast::<MsvcString>().as_ref() }) else {
            return false;
        };
        self.note_route(text.as_bytes(), via);
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
