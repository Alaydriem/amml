use std::ffi::c_void;
use std::sync::atomic::{AtomicPtr, Ordering};

use crate::{Boundary, SessionHook, TextureReloadAdapter};

/// `void (MinecraftGame*, ResourcePackManager&, bool changed)`, the function that applies the active resource pack
/// stack: rcx the game, rdx the manager, r8 the bool, passed on as the whole register.
type ApplyFn = unsafe extern "C" fn(*mut c_void, *mut c_void, u64);

static APPLY: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());

/// The loader's detour on `MinecraftGame.applyActiveResourcePacks`: it calls the original once with the arguments it
/// received, bit for bit, and then runs the adapter inside `Boundary::run`.
pub struct TextureReloadDetours;

impl TextureReloadDetours {
    pub const APPLY: &'static str = "MinecraftGame.applyActiveResourcePacks";

    /// The one hook.
    pub fn hook() -> SessionHook {
        SessionHook {
            entry: Self::APPLY,
            vtable: None,
            detour: Self::apply as ApplyFn as *const c_void,
            original: &APPLY,
        }
    }

    unsafe extern "C" fn apply(game: *mut c_void, manager: *mut c_void, changed: u64) {
        const { assert!(std::mem::size_of::<ApplyFn>() == std::mem::size_of::<*mut c_void>()) };
        let address = APPLY.load(Ordering::Acquire);
        // SAFETY: the engine set the slot before it moved the thunk to this detour, and the hooked function has the
        // signature `ApplyFn`, which is pointer sized.
        let original = unsafe { std::mem::transmute_copy::<*mut c_void, ApplyFn>(&address) };
        // SAFETY: the game's own arguments, passed on unchanged.
        unsafe { original(game, manager, changed) };
        Boundary::run((), || TextureReloadAdapter::with((), TextureReloadAdapter::pack_changed));
    }
}
