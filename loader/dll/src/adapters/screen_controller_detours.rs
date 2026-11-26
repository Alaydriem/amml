use std::ffi::c_void;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicPtr, Ordering};

use crate::{Boundary, HudAdapter, LoaderState, ScreenAdapter, SessionHook};

/// `ScreenController::~ScreenController()` (the body every screen controller's destructor reaches): rcx the
/// controller.
type DtorFn = unsafe extern "C" fn(*mut c_void);

static DTOR: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
static INSTALLED: OnceLock<bool> = OnceLock::new();

/// The loader's one detour on the screen controller destructor. Before the game frees a controller it tells the
/// HUD adapter (which forgets its HUD controller) and the screen adapter (which retires the screen and raises
/// `ScreenClosing`), then calls the original exactly once with the register it received.
pub struct ScreenControllerDetours;

impl ScreenControllerDetours {
    pub const DTOR: &'static str = "ScreenController.dtor";

    /// Installs the detour once per process, for whichever adapter needs it first; every later call gives the
    /// result of the first one. False, with a log line, when the target is unknown or the hook is refused.
    pub(crate) fn install(state: &'static LoaderState, priority: i32) -> bool {
        *INSTALLED.get_or_init(|| Self::hook().install(state, priority))
    }

    fn hook() -> SessionHook {
        SessionHook { entry: Self::DTOR, vtable: None, detour: Self::dtor as DtorFn as *const c_void, original: &DTOR }
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

    unsafe extern "C" fn dtor(this: *mut c_void) {
        Boundary::run((), || {
            HudAdapter::with(|adapter| adapter.forget(this));
            ScreenAdapter::with((), |adapter| adapter.closing(this));
        });
        // SAFETY: the hook on the screen controller destructor is installed, so its slot holds a `DtorFn`.
        let original = unsafe { Self::original::<DtorFn>(&DTOR) };
        // SAFETY: the game's own argument, passed on unchanged.
        unsafe { original(this) };
    }
}
