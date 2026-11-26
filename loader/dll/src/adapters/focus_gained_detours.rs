use std::ffi::c_void;
use std::sync::atomic::{AtomicPtr, AtomicU32, Ordering};

use crate::{Boundary, FocusAdapter, ScreenDetours, SessionHook};

/// `void MinecraftGame::onAppFocusGained()`: rcx the AppPlatformListener subobject, no other argument, no result.
type ObjectFn = unsafe extern "C" fn(*mut c_void);

static ORIGINAL: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
// What the focus detours have work for besides the game's own call. Zero: each detour calls the original and nothing
// else.
static WANTED: AtomicU32 = AtomicU32::new(0);

/// The loader's detour on `MinecraftGame::onAppFocusGained` (the slot `MinecraftGame.onAppFocusGained` of the
/// `MinecraftGame.AppPlatformListener.vtable` global) and the flags word that this detour and the focus-loss detour of
/// `ClientDetours` read.
///
/// With no flag set, the detour calls the original and does nothing else: one relaxed load, no lock, no allocation, no
/// log. `LOSS_LISTENER` is set while a mod listens to `AppFocusLost`; `GAIN_LISTENER` while a mod listens to
/// `AppFocusGained`; `HELD` while a mod holds a loss it cancelled. A panic inside the loader's work is caught at the
/// boundary; the game's own call runs once with its own argument.
pub struct FocusGainedDetours;

impl FocusGainedDetours {
    pub const ENTRY: &'static str = "MinecraftGame.onAppFocusGained";
    pub const LISTENER_VTABLE: &'static str = "MinecraftGame.AppPlatformListener.vtable";
    /// A mod listens to `AppFocusLost`: the loss detour raises it Before the game's handler.
    pub const LOSS_LISTENER: u32 = 1;
    /// A mod listens to `AppFocusGained`: the gain detour raises it After the game's handler.
    pub const GAIN_LISTENER: u32 = 2;
    /// A mod holds a loss it cancelled: the gain detour forgets it.
    pub const HELD: u32 = 4;

    /// The hook on the slot.
    pub fn hook() -> SessionHook {
        SessionHook {
            entry: Self::ENTRY,
            vtable: Some(Self::LISTENER_VTABLE),
            detour: Self::gained as ObjectFn as *const c_void,
            original: &ORIGINAL,
        }
    }

    /// Sets or clears `flag`.
    pub fn set(flag: u32, on: bool) {
        if on {
            WANTED.fetch_or(flag, Ordering::AcqRel);
        } else {
            WANTED.fetch_and(!flag, Ordering::AcqRel);
        }
    }

    /// The flags word: one relaxed load.
    pub fn flags() -> u32 {
        WANTED.load(Ordering::Relaxed)
    }

    unsafe extern "C" fn gained(this: *mut c_void) {
        // SAFETY: the hook on `MinecraftGame.onAppFocusGained` is installed, so its slot holds an `ObjectFn`.
        let original = unsafe { ScreenDetours::original::<ObjectFn>(&ORIGINAL) };
        // SAFETY: the game's own argument, passed on unchanged.
        unsafe { original(this) };
        if Self::flags() & (Self::GAIN_LISTENER | Self::HELD) != 0 {
            Boundary::run((), || FocusAdapter::with((), FocusAdapter::gained));
        }
    }
}
