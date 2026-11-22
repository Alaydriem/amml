use std::ffi::c_void;
use std::hint::black_box;
use std::sync::Mutex;

use amml_data::{EntryKind, Resolved};

use super::{ClientGame, Harness, RouteText};

// Each stand-in router call as a line: `push <route> mode=<mode>` or `replace <route>`.
static CALLS: Mutex<Vec<String>> = Mutex::new(Vec::new());
// The routes the stand-in router refuses (gives false for), as the game's router does for a route it does not take.
static REFUSED: Mutex<Vec<String>> = Mutex::new(Vec::new());

/// Stand-ins with the game signatures for `OreUI::Router::_pushRoute` and `replaceRoute`: each records the route and
/// gives `true` (1).
pub struct RouterGame;

impl RouterGame {
    pub const PUSH_ROUTE: &'static str = "OreUIRouter.pushRoute";
    pub const REPLACE_ROUTE: &'static str = "OreUIRouter.replaceRoute";
    const ROUTER: usize = 0x7300;

    /// The client entries, the client's current router (slot 11) and the two router functions; `without` names
    /// entries to leave out.
    pub fn entries(without: &[&str]) -> Vec<(&'static str, Resolved)> {
        let function = |address: *const ()| Resolved {
            kind: EntryKind::Function,
            value: address as usize as u64 - Harness::BASE,
            recipes_agreeing: 2,
        };
        let mut entries = ClientGame::entries(&[]);
        entries.extend([
            ("ClientInstance.getCurrentUIRouter", Resolved { kind: EntryKind::VSlot, value: 11, recipes_agreeing: 2 }),
            (Self::PUSH_ROUTE, function(Self::push_route as *const ())),
            (Self::REPLACE_ROUTE, function(Self::replace_route as *const ())),
        ]);
        entries.into_iter().filter(|(id, _)| !without.contains(id)).collect()
    }

    /// Calls (through its address) `_pushRoute(router, route, mode)` with an MSVC string of `route`; gives the bool.
    pub fn push(route: &str, mode: u32) -> u8 {
        let text = RouteText::new(route);
        let function = black_box(Self::push_route as extern "C" fn(*mut c_void, *const RouteText, u64) -> u8);
        function(black_box(Self::ROUTER as *mut c_void), black_box(&raw const text), black_box(u64::from(mode)))
    }

    /// Calls (through its address) `replaceRoute(router, route)`; gives the bool.
    pub fn replace(route: &str) -> u8 {
        let text = RouteText::new(route);
        let function = black_box(Self::replace_route as extern "C" fn(*mut c_void, *const RouteText) -> u8);
        function(black_box(Self::ROUTER as *mut c_void), black_box(&raw const text))
    }

    /// From now on the stand-in router refuses `route` (gives false).
    pub fn refuse(route: &str) {
        REFUSED.lock().unwrap().push(route.to_string());
    }

    fn answer(route: &str) -> u8 {
        u8::from(!REFUSED.lock().unwrap().iter().any(|refused| refused == route))
    }

    /// The stand-in calls so far.
    pub fn calls() -> Vec<String> {
        CALLS.lock().unwrap().clone()
    }

    fn text(route: *const RouteText) -> String {
        RouteText::read(route)
    }

    /// `OreUI::Router* ClientInstance::getCurrentUIRouter()`: the stand-in router address.
    #[inline(never)]
    pub extern "C" fn current_router(this: *mut c_void) -> usize {
        black_box(this);
        Self::ROUTER
    }

    #[inline(never)]
    extern "C" fn push_route(this: *mut c_void, route: *const RouteText, mode: u64) -> u8 {
        black_box(this);
        let text = Self::text(route);
        CALLS.lock().unwrap().push(format!("push {text} mode={}", black_box(mode)));
        Self::answer(&text)
    }

    #[inline(never)]
    extern "C" fn replace_route(this: *mut c_void, route: *const RouteText) -> u8 {
        black_box(this);
        let text = Self::text(route);
        CALLS.lock().unwrap().push(format!("replace {text}"));
        Self::answer(&text)
    }
}
