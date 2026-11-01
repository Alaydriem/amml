//! The detour on `ContainerScreenController::_onContainerSlotHovered`.

use amml_msvc::MsvcString;
use amml_sdk::DetourGuard;

use crate::PEEK;

/// The original slot of the hook.
pub static GUARD: DetourGuard = DetourGuard::new("slot_hovered");

/// The signature of the hooked function: `this`, the collection name, the slot, and the `ui::ViewRequest` in EAX.
pub type SlotHoveredFn = unsafe extern "C" fn(usize, usize, i32) -> u32;

// ui::ViewRequest ContainerScreenController::_onContainerSlotHovered(std::string const&, int). The original runs
// first; the probe reads the slot it stored and returns the original's value unchanged.
pub extern "C" fn detour(this: usize, collection: usize, slot: i32) -> u32 {
    let original: SlotHoveredFn = GUARD.original_fn();
    // SAFETY: `original` is the next function in the chain, with the hooked signature; the arguments are the game's.
    #[allow(unsafe_code)]
    let result = unsafe { original(this, collection, slot) };
    PEEK.on_hover(this, collection as *const MsvcString, slot);
    result
}
