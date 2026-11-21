use std::ffi::c_void;
use std::sync::atomic::{AtomicPtr, Ordering};

use amml_abi::AmmlRouteVia;

use crate::{Boundary, RouterAdapter, SessionHook};

/// `bool OreUI::Router::_pushRoute(std::string const& route, RouterPushMode mode)`: rcx the router, rdx the route,
/// r8 the mode, passed on as the whole register; the bool in al.
type PushRouteFn = unsafe extern "C" fn(*mut c_void, *const c_void, u64) -> u8;
/// `bool OreUI::Router::replaceRoute(std::string const& route)`: rcx the router, rdx the route; the bool in al.
type ReplaceRouteFn = unsafe extern "C" fn(*mut c_void, *const c_void) -> u8;

static PUSH_ROUTE: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
static REPLACE_ROUTE: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());

/// The loader's detours on `OreUIRouter.pushRoute` and `OreUIRouter.replaceRoute`.
///
/// Each detour asks the router adapter inside `Boundary::run` whether the route is refused; a panic there lets the
/// route through. A refused route returns `false` (0) to the game without calling the original; every other call
/// goes to the original once, with the game's arguments unchanged.
pub struct RouterDetours;

impl RouterDetours {
    pub const PUSH_ROUTE: &'static str = "OreUIRouter.pushRoute";
    pub const REPLACE_ROUTE: &'static str = "OreUIRouter.replaceRoute";

    /// The two hooks, in install order.
    pub fn hooks() -> [SessionHook; 2] {
        [
            SessionHook {
                entry: Self::PUSH_ROUTE,
                vtable: None,
                detour: Self::push_route as PushRouteFn as *const c_void,
                original: &PUSH_ROUTE,
            },
            SessionHook {
                entry: Self::REPLACE_ROUTE,
                vtable: None,
                detour: Self::replace_route as ReplaceRouteFn as *const c_void,
                original: &REPLACE_ROUTE,
            },
        ]
    }

    // The original behind `slot`, as a function of type `F`.
    //
    // Safety: the hook of `slot` is installed and its target has the signature `F`.
    unsafe fn original<F: Copy>(slot: &AtomicPtr<c_void>) -> F {
        const { assert!(std::mem::size_of::<F>() == std::mem::size_of::<*mut c_void>()) };
        let address = slot.load(Ordering::Acquire);
        // SAFETY: the engine set the slot before it moved the thunk to the detour that calls this; the caller
        // guarantees the signature, which is pointer sized.
        unsafe { std::mem::transmute_copy::<*mut c_void, F>(&address) }
    }

    fn refused(route: *const c_void, via: AmmlRouteVia) -> bool {
        Boundary::run(false, || {
            RouterAdapter::with(false, |adapter| {
                // SAFETY: the router passes a live `std::string const&` (or null) for the whole call.
                unsafe { adapter.refuses(route, via) }
            })
        })
    }

    unsafe extern "C" fn push_route(this: *mut c_void, route: *const c_void, mode: u64) -> u8 {
        if Self::refused(route, AmmlRouteVia::PUSH) {
            return 0;
        }
        // SAFETY: the hook on `OreUIRouter.pushRoute` is installed, so its slot holds a `PushRouteFn`.
        let original = unsafe { Self::original::<PushRouteFn>(&PUSH_ROUTE) };
        // SAFETY: the game's own arguments, passed on unchanged.
        unsafe { original(this, route, mode) }
    }

    unsafe extern "C" fn replace_route(this: *mut c_void, route: *const c_void) -> u8 {
        if Self::refused(route, AmmlRouteVia::REPLACE) {
            return 0;
        }
        // SAFETY: the hook on `OreUIRouter.replaceRoute` is installed, so its slot holds a `ReplaceRouteFn`.
        let original = unsafe { Self::original::<ReplaceRouteFn>(&REPLACE_ROUTE) };
        // SAFETY: the game's own arguments, passed on unchanged.
        unsafe { original(this, route) }
    }
}
