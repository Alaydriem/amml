//! The detour on `ScreenController::~ScreenController`.

use amml_sdk::DetourGuard;

use crate::PEEK;

/// The original slot of the hook.
pub static GUARD: DetourGuard = DetourGuard::new("controller_dtor");

/// The signature of the hooked function: `this` only.
pub type ControllerDtorFn = unsafe extern "C" fn(usize);

// void ScreenController::~ScreenController(). The probe runs first, because the controller is gone after the
// original; it reads nothing through `this`, only compares the address.
pub extern "C" fn detour(this: usize) {
    PEEK.on_controller_dtor(this);
    let original: ControllerDtorFn = GUARD.original_fn();
    // SAFETY: `original` is the next function in the chain, with the hooked signature; `this` is the game's.
    #[allow(unsafe_code)]
    unsafe {
        original(this)
    };
}
