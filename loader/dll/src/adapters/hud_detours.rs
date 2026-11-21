use std::ffi::c_void;
use std::sync::atomic::{AtomicPtr, Ordering};

use crate::{Boundary, HudAdapter, SessionHook};

/// `ui::DirtyFlag HudScreenController::tick()`: rcx the controller, the flag (a 32-bit enum) in eax.
type TickFn = unsafe extern "C" fn(*mut c_void) -> u32;

static TICK: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());

/// The loader's detour on the HUD screen controller tick, which notes the controller. It calls the original exactly
/// once with the registers it received. The detour on the screen controller destructor, which makes the HUD adapter
/// forget the controller before it is freed, is `ScreenControllerDetours`.
pub struct HudDetours;

impl HudDetours {
    pub const TICK: &'static str = "HudScreenController.tick";
    pub const VTABLE: &'static str = "HudScreenController.vtable";

    /// The hooks, in install order.
    pub fn hooks() -> [SessionHook; 1] {
        [SessionHook {
            entry: Self::TICK,
            vtable: Some(Self::VTABLE),
            detour: Self::tick as TickFn as *const c_void,
            original: &TICK,
        }]
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

    unsafe extern "C" fn tick(this: *mut c_void) -> u32 {
        Boundary::run((), || HudAdapter::with(|adapter| adapter.note_tick(this)));
        // SAFETY: the hook on the HUD tick is installed, so its slot holds a `TickFn`.
        let original = unsafe { Self::original::<TickFn>(&TICK) };
        // SAFETY: the game's own argument, passed on unchanged.
        unsafe { original(this) }
    }
}
