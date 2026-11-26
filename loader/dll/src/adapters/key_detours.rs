use std::ffi::c_void;
use std::sync::atomic::{AtomicPtr, Ordering};

use crate::{Boundary, HidKeyFeed, KeyFeed, ProcessorKeyFeed, SessionHook};

/// `void HIDControllerGameCoreDesktop::onKeyDown(int keyCode, InputOrigin origin)`: rcx the controller, edx
/// the key, r8d the origin; rdx and r8 are passed on as whole registers.
type KeyDownFn = unsafe extern "C" fn(*mut c_void, u64, u64);
/// `void HIDControllerGameCoreDesktop::onKeyUp(int keyCode)`: rcx the controller, edx the key, passed on as
/// the whole register.
type KeyUpFn = unsafe extern "C" fn(*mut c_void, u64);
/// `void HIDControllerGameCoreDesktop::update()`: rcx the controller.
type UpdateFn = unsafe extern "C" fn(*mut c_void);
/// `void HIDController::resetKeys()`: rcx the controller.
type ResetKeysFn = unsafe extern "C" fn(*mut c_void);
/// `void KeyboardEventProcessor::_feedKeyState(int key, int state)`: rcx (never read by the body), edx the
/// key, r8d the state (1 down, 0 up); all three are passed on as whole registers.
type FeedKeyStateFn = unsafe extern "C" fn(*mut c_void, u64, u64);

static KEY_DOWN: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
static KEY_UP: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
static UPDATE: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
static RESET_KEYS: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
static FEED_KEY_STATE: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());

/// The loader's detours on the game's key functions, and the loader's own calls past them.
///
/// Each detour runs the real key path inside `Boundary::run` (a panic there sends the key on) and then
/// calls the original exactly once, with the registers it received, unless the key path holds the key
/// back. The loader's own calls (`send_key_down` and the others) call the original slot directly, so
/// they raise no event and do not touch the key table.
pub struct KeyDetours;

impl KeyDetours {
    pub const KEY_DOWN: &'static str = "HIDControllerGameCoreDesktop.onKeyDown";
    pub const KEY_UP: &'static str = "HIDControllerGameCoreDesktop.onKeyUp";
    pub const UPDATE: &'static str = "HIDControllerGameCoreDesktop.update";
    pub const RESET_KEYS: &'static str = "HIDController.resetKeys";
    pub const FEED_KEY_STATE: &'static str = "KeyboardEventProcessor.feedKeyState";

    /// The three hooks of the `hid` shape, in install order.
    pub fn hid_hooks() -> [SessionHook; 3] {
        [
            SessionHook {
                entry: Self::KEY_DOWN,
                vtable: None,
                detour: Self::key_down as KeyDownFn as *const c_void,
                original: &KEY_DOWN,
            },
            SessionHook {
                entry: Self::KEY_UP,
                vtable: None,
                detour: Self::key_up as KeyUpFn as *const c_void,
                original: &KEY_UP,
            },
            SessionHook {
                entry: Self::UPDATE,
                vtable: None,
                detour: Self::update as UpdateFn as *const c_void,
                original: &UPDATE,
            },
        ]
    }

    /// The optional hook of the `hid` shape: the key reset of a focus loss.
    pub fn reset_hook() -> SessionHook {
        SessionHook {
            entry: Self::RESET_KEYS,
            vtable: None,
            detour: Self::reset_keys as ResetKeysFn as *const c_void,
            original: &RESET_KEYS,
        }
    }

    /// The hook of the `key_processor` shape.
    pub fn processor_hook() -> SessionHook {
        SessionHook {
            entry: Self::FEED_KEY_STATE,
            vtable: None,
            detour: Self::feed_key_state as FeedKeyStateFn as *const c_void,
            original: &FEED_KEY_STATE,
        }
    }

    /// Calls `onKeyDown(controller, key, origin)` past the loader's detour; false while the hook is not
    /// installed.
    ///
    /// # Safety
    /// `controller` is the live keyboard controller the game passed to its own key functions.
    pub(crate) unsafe fn send_key_down(controller: *mut c_void, key: u16, origin: i32) -> bool {
        let Some(original) = Self::installed(&KEY_DOWN) else {
            return false;
        };
        // SAFETY: the slot holds the next hook or the trampoline of a hooked `KeyDownFn`.
        let original = unsafe { std::mem::transmute::<*mut c_void, KeyDownFn>(original) };
        // SAFETY: the caller guarantees `controller`; the origin goes in r8d as the game passes it.
        unsafe { original(controller, u64::from(key), u64::from(origin as u32)) };
        true
    }

    /// Calls `onKeyUp(controller, key)` past the loader's detour; false while the hook is not installed.
    ///
    /// # Safety
    /// As `send_key_down`.
    pub(crate) unsafe fn send_key_up(controller: *mut c_void, key: u16) -> bool {
        let Some(original) = Self::installed(&KEY_UP) else {
            return false;
        };
        // SAFETY: the slot holds the next hook or the trampoline of a hooked `KeyUpFn`.
        let original = unsafe { std::mem::transmute::<*mut c_void, KeyUpFn>(original) };
        // SAFETY: the caller guarantees `controller`.
        unsafe { original(controller, u64::from(key)) };
        true
    }

    /// Calls `_feedKeyState(null, key, state)` past the loader's detour; false while the hook is not
    /// installed. The body never reads `this`.
    pub(crate) fn send_key_state(key: u16, down: bool) -> bool {
        let Some(original) = Self::installed(&FEED_KEY_STATE) else {
            return false;
        };
        // SAFETY: the slot holds the next hook or the trampoline of a hooked `FeedKeyStateFn`.
        let original = unsafe { std::mem::transmute::<*mut c_void, FeedKeyStateFn>(original) };
        // SAFETY: the function does not read rcx; a key below 100h indexes its own 256-entry table.
        unsafe { original(std::ptr::null_mut(), u64::from(key), u64::from(down)) };
        true
    }

    // The next hook or trampoline in `slot`; `None` while it is null.
    fn installed(slot: &AtomicPtr<c_void>) -> Option<*mut c_void> {
        Some(slot.load(Ordering::Acquire)).filter(|address| !address.is_null())
    }

    // The original behind `slot`, as a function of type `F`.
    //
    // Safety: the hook of `slot` is installed and its target has the signature `F`.
    unsafe fn original<F: Copy>(slot: &AtomicPtr<c_void>) -> F {
        const { assert!(std::mem::size_of::<F>() == std::mem::size_of::<*mut c_void>()) };
        let address = slot.load(Ordering::Acquire);
        // SAFETY: the engine set the slot before it moved the thunk to the detour that calls this; the
        // caller guarantees the signature, which is pointer sized.
        unsafe { std::mem::transmute_copy::<*mut c_void, F>(&address) }
    }

    // The low 32 bits of a register that holds an `int` key, as a key code.
    fn key_of(register: u64) -> Option<u16> {
        u16::try_from(register as u32).ok()
    }

    unsafe extern "C" fn key_down(this: *mut c_void, key: u64, origin: u64) {
        let send = Boundary::run(true, || match Self::key_of(key) {
            Some(code) => HidKeyFeed::with(true, |feed| feed.real_down(this, code, origin as u32 as i32)),
            None => true,
        });
        if !send {
            return;
        }
        // SAFETY: the hook on `onKeyDown` is installed, so its slot holds a `KeyDownFn`.
        let original = unsafe { Self::original::<KeyDownFn>(&KEY_DOWN) };
        // SAFETY: the game's own registers, passed on unchanged.
        unsafe { original(this, key, origin) };
    }

    unsafe extern "C" fn key_up(this: *mut c_void, key: u64) {
        let send = Boundary::run(true, || match Self::key_of(key) {
            Some(code) => HidKeyFeed::with(true, |feed| feed.real_up(this, code)),
            None => true,
        });
        if !send {
            return;
        }
        // SAFETY: the hook on `onKeyUp` is installed, so its slot holds a `KeyUpFn`.
        let original = unsafe { Self::original::<KeyUpFn>(&KEY_UP) };
        // SAFETY: the game's own registers, passed on unchanged.
        unsafe { original(this, key) };
    }

    unsafe extern "C" fn update(this: *mut c_void) {
        Boundary::run((), || HidKeyFeed::with((), |feed| feed.note_controller(this)));
        // SAFETY: the hook on `update` is installed, so its slot holds an `UpdateFn`.
        let original = unsafe { Self::original::<UpdateFn>(&UPDATE) };
        // SAFETY: the game's own argument, passed on unchanged.
        unsafe { original(this) };
    }

    // The game resets its keys whatever the listeners answer, so the original always runs.
    unsafe extern "C" fn reset_keys(this: *mut c_void) {
        Boundary::run((), || HidKeyFeed::with((), |feed| feed.release_real()));
        // SAFETY: the hook on the key reset is installed, so its slot holds a `ResetKeysFn`.
        let original = unsafe { Self::original::<ResetKeysFn>(&RESET_KEYS) };
        // SAFETY: the game's own argument, passed on unchanged.
        unsafe { original(this) };
    }

    unsafe extern "C" fn feed_key_state(this: *mut c_void, key: u64, state: u64) {
        let send = Boundary::run(true, || match Self::key_of(key) {
            Some(code) => ProcessorKeyFeed::with(true, |feed| feed.real(code, state as u32 != 0)),
            None => true,
        });
        if !send {
            return;
        }
        // SAFETY: the hook on `_feedKeyState` is installed, so its slot holds a `FeedKeyStateFn`.
        let original = unsafe { Self::original::<FeedKeyStateFn>(&FEED_KEY_STATE) };
        // SAFETY: the game's own registers, passed on unchanged.
        unsafe { original(this, key, state) };
    }
}
