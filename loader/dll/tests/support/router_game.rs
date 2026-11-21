use std::ffi::c_void;
use std::hint::black_box;
use std::sync::Mutex;

use amml_data::{EntryKind, Resolved};

use super::{ClientGame, Harness, RouteText};

// Each stand-in router call as a line: `push <route> mode=<mode>` or `replace <route>`.
static CALLS: Mutex<Vec<String>> = Mutex::new(Vec::new());

/// Stand-ins with the game signatures for `OreUI::Router::_pushRoute` and `replaceRoute`: each records the route and
/// gives `true` (1).
pub struct RouterGame;

impl RouterGame {
    pub const PUSH_ROUTE: &'static str = "OreUIRouter.pushRoute";
    pub const REPLACE_ROUTE: &'static str = "OreUIRouter.replaceRoute";
    const ROUTER: usize = 0x7300;

    /// The client entries and the two router functions; `without` names entries to leave out.
    pub fn entries(without: &[&str]) -> Vec<(&'static str, Resolved)> {
        let function = |address: *const ()| Resolved {
            kind: EntryKind::Function,
            value: address as usize as u64 - Harness::BASE,
            recipes_agreeing: 2,
        };
        let mut entries = ClientGame::entries(&[]);
        entries.extend([
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

    /// The stand-in calls so far.
    pub fn calls() -> Vec<String> {
        CALLS.lock().unwrap().clone()
    }

    fn text(route: *const RouteText) -> String {
        RouteText::read(route)
    }

    #[inline(never)]
    extern "C" fn push_route(this: *mut c_void, route: *const RouteText, mode: u64) -> u8 {
        black_box(this);
        CALLS.lock().unwrap().push(format!("push {} mode={}", Self::text(route), black_box(mode)));
        1
    }

    #[inline(never)]
    extern "C" fn replace_route(this: *mut c_void, route: *const RouteText) -> u8 {
        black_box(this);
        CALLS.lock().unwrap().push(format!("replace {}", Self::text(route)));
        1
    }
}
