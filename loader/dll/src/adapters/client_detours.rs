use std::ffi::c_void;
use std::sync::atomic::{AtomicPtr, Ordering};

use amml_data::AdapterCatalog;

use crate::{Boundary, ClientAdapter, SessionHook};

/// `void MinecraftGame::grabMouse()`, `void IMinecraftGame::releaseMouse()` and
/// `void MinecraftGame::onAppFocusLost()`: rcx the object (for `onAppFocusLost`, the AppPlatformListener
/// subobject), no other argument, no result.
type ObjectFn = unsafe extern "C" fn(*mut c_void);

static GRAB: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
static RELEASE: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
static FOCUS_LOST: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());

/// The loader's detours on the mouse grab and focus functions of `MinecraftGame`, and the loader's own
/// calls past them.
///
/// The grab hook patches the prologue of the `MinecraftGame.grabMouse` function. The release hook reads
/// the function in slot `IMinecraftGame.releaseMouse` of the `MinecraftGame.vtable` global (the
/// IMinecraftGame vtable of MinecraftGame, stored at MinecraftGame+0) and patches that function's
/// prologue; no vtable is written. The focus hook patches the prologue of `MinecraftGame.onAppFocusLost`.
///
/// Each detour calls the original exactly once unless a listener cancelled the grab, and raises inside
/// `Boundary::run`, so a panic while raising never skips the original. The loader's own calls
/// (`grab_past_detour` and the others) call the original slot directly, so they raise no event and change
/// no state in the detours.
pub struct ClientDetours;

impl ClientDetours {
    pub const GRAB: &'static str = "MinecraftGame.grabMouse";
    pub const RELEASE: &'static str = "IMinecraftGame.releaseMouse";
    pub const FOCUS_LOST: &'static str = "MinecraftGame.onAppFocusLost";

    /// The three hooks, in install order.
    pub fn hooks() -> [SessionHook; 3] {
        [
            SessionHook {
                entry: Self::GRAB,
                vtable: None,
                detour: Self::grab as ObjectFn as *const c_void,
                original: &GRAB,
            },
            SessionHook {
                entry: Self::RELEASE,
                vtable: Some(AdapterCatalog::MINECRAFT_GAME_VTABLE),
                detour: Self::release as ObjectFn as *const c_void,
                original: &RELEASE,
            },
            SessionHook {
                entry: Self::FOCUS_LOST,
                vtable: None,
                detour: Self::focus_lost as ObjectFn as *const c_void,
                original: &FOCUS_LOST,
            },
        ]
    }

    /// Calls `MinecraftGame::grabMouse` on `game` past the loader's detour; false when the hook is not
    /// installed.
    ///
    /// # Safety
    /// `game` is a live MinecraftGame whose vtable is the `MinecraftGame.vtable` global.
    pub(crate) unsafe fn grab_past_detour(game: *mut c_void) -> bool {
        // SAFETY: the caller guarantees `game`.
        unsafe { Self::call_original(&GRAB, game) }
    }

    /// Calls `MinecraftGame::releaseMouse` on `game` past the loader's detour; false when the hook is not
    /// installed.
    ///
    /// # Safety
    /// As `grab_past_detour`.
    pub(crate) unsafe fn release_past_detour(game: *mut c_void) -> bool {
        // SAFETY: the caller guarantees `game`.
        unsafe { Self::call_original(&RELEASE, game) }
    }

    /// Calls `MinecraftGame::onAppFocusLost` on `listener` past the loader's detour; false when the hook
    /// is not installed.
    ///
    /// # Safety
    /// `listener` is the AppPlatformListener subobject of a live MinecraftGame.
    pub(crate) unsafe fn focus_lost_past_detour(listener: *mut c_void) -> bool {
        // SAFETY: the caller guarantees `listener`.
        unsafe { Self::call_original(&FOCUS_LOST, listener) }
    }

    // Calls the function in `slot` (the next hook or the trampoline) on `this`; false while it is null.
    //
    // Safety: `this` is a valid `this` for the hooked function behind `slot`.
    unsafe fn call_original(slot: &AtomicPtr<c_void>, this: *mut c_void) -> bool {
        let address = slot.load(Ordering::Acquire);
        if address.is_null() {
            return false;
        }
        // SAFETY: the engine set the slot to the next hook or the trampoline of a hooked `ObjectFn`.
        let original = unsafe { std::mem::transmute::<*mut c_void, ObjectFn>(address) };
        // SAFETY: the caller guarantees `this`.
        unsafe { original(this) };
        true
    }

    // The original behind `slot`. Safety: the hook of `slot` is installed.
    unsafe fn original(slot: &AtomicPtr<c_void>) -> ObjectFn {
        let address = slot.load(Ordering::Acquire);
        // SAFETY: the engine set the slot before it moved the thunk to the detour that calls this.
        unsafe { std::mem::transmute::<*mut c_void, ObjectFn>(address) }
    }

    unsafe extern "C" fn grab(this: *mut c_void) {
        let proceed = Boundary::run(true, || ClientAdapter::with(true, ClientAdapter::grab_requested));
        if !proceed {
            return;
        }
        // SAFETY: the hook on `MinecraftGame.grabMouse` is installed, so its slot holds an `ObjectFn`.
        let original = unsafe { Self::original(&GRAB) };
        // SAFETY: the game's own argument, passed on unchanged.
        unsafe { original(this) };
        Boundary::run((), || ClientAdapter::with((), |adapter| adapter.note_captured(true)));
    }

    unsafe extern "C" fn release(this: *mut c_void) {
        // SAFETY: the hook on `IMinecraftGame.releaseMouse` is installed, so its slot holds an `ObjectFn`.
        let original = unsafe { Self::original(&RELEASE) };
        // SAFETY: the game's own argument, passed on unchanged.
        unsafe { original(this) };
        Boundary::run((), || ClientAdapter::with((), |adapter| adapter.note_captured(false)));
    }

    unsafe extern "C" fn focus_lost(this: *mut c_void) {
        // SAFETY: the hook on `MinecraftGame.onAppFocusLost` is installed, so its slot holds an `ObjectFn`.
        let original = unsafe { Self::original(&FOCUS_LOST) };
        // SAFETY: the game's own argument, passed on unchanged.
        unsafe { original(this) };
        Boundary::run((), || ClientAdapter::with((), ClientAdapter::focus_lost));
    }
}
