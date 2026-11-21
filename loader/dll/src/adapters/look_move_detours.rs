use std::ffi::c_void;
use std::sync::atomic::{AtomicPtr, Ordering};

use crate::{Boundary, GamepadAdapter, LookAdapter, MoveAdapter, SessionHook};

/// `void LocalPlayer::_applyTurnDelta(Vec2 const& turnOffset)`: rcx the player, rdx the `Vec2` (x pitch, y
/// yaw, in degrees).
type ApplyTurnDeltaFn = unsafe extern "C" fn(*mut c_void, *const [f32; 2]);
/// `static void ClientInputUpdateSystem::extractRawHIDInput(MovementAbilitiesComponent const&,
/// MoveInputComponent const&, ActorDataFlagComponent const&, RawMoveInputComponent&,
/// Optional<SneakingComponent const>, Optional<WasInWaterFlagComponent const>)`: rcx, rdx, r8, r9 (the raw
/// component) and the stack slots at +28h and +30h. Every register and slot is passed on whole.
type ExtractRawHidInputFn = unsafe extern "C" fn(u64, u64, u64, *mut c_void, u64, u64);
/// `void GameControllerHandler_GameCore::refresh()`: rcx the handler.
type RefreshFn = unsafe extern "C" fn(*mut c_void);

static APPLY_TURN_DELTA: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
static EXTRACT_RAW_HID_INPUT: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
static REFRESH: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());

/// The loader's detours on the game's turn, move and gamepad functions, and the loader's own turn call
/// past them.
///
/// The turn and gamepad detours run their adapter inside `Boundary::run` (a panic there calls the
/// original unchanged) and call the original once unless the event was cancelled. The move detour
/// calls the original first and then runs its adapter on the component, inside `Boundary::run`.
pub struct LookMoveDetours;

impl LookMoveDetours {
    pub const APPLY_TURN_DELTA: &'static str = "LocalPlayer.applyTurnDelta";
    pub const EXTRACT_RAW_HID_INPUT: &'static str = "ClientInputUpdateSystem.extractRawHIDInput";
    pub const REFRESH: &'static str = "GameControllerHandler_GameCore.refresh";

    pub fn turn_hook() -> SessionHook {
        SessionHook {
            entry: Self::APPLY_TURN_DELTA,
            vtable: None,
            detour: Self::apply_turn_delta as ApplyTurnDeltaFn as *const c_void,
            original: &APPLY_TURN_DELTA,
        }
    }

    pub fn move_hook() -> SessionHook {
        SessionHook {
            entry: Self::EXTRACT_RAW_HID_INPUT,
            vtable: None,
            detour: Self::extract_raw_hid_input as ExtractRawHidInputFn as *const c_void,
            original: &EXTRACT_RAW_HID_INPUT,
        }
    }

    pub fn gamepad_hook() -> SessionHook {
        SessionHook {
            entry: Self::REFRESH,
            vtable: None,
            detour: Self::refresh as RefreshFn as *const c_void,
            original: &REFRESH,
        }
    }

    /// Calls `_applyTurnDelta(player, &[pitch, yaw])` past the loader's detour; false while the hook is
    /// not installed.
    ///
    /// # Safety
    /// `player` is the live local player of the primary client, and the call runs on the game thread.
    pub(crate) unsafe fn send_turn(player: *mut c_void, pitch: f32, yaw: f32) -> bool {
        let Some(original) = Self::installed(&APPLY_TURN_DELTA) else {
            return false;
        };
        // SAFETY: the slot holds the next hook or the trampoline of a hooked `ApplyTurnDeltaFn`.
        let original = unsafe { std::mem::transmute::<*mut c_void, ApplyTurnDeltaFn>(original) };
        let delta = [pitch, yaw];
        // SAFETY: the caller guarantees `player` and the thread; `delta` lives for the call.
        unsafe { original(player, &delta) };
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

    unsafe extern "C" fn apply_turn_delta(this: *mut c_void, delta: *const [f32; 2]) {
        // SAFETY: the hook on `_applyTurnDelta` is installed, so its slot holds an `ApplyTurnDeltaFn`.
        let original = unsafe { Self::original::<ApplyTurnDeltaFn>(&APPLY_TURN_DELTA) };
        if delta.is_null() {
            // SAFETY: the game's own arguments.
            unsafe { original(this, delta) };
            return;
        }
        // SAFETY: the game passes a live `Vec2 const&` for the call; it is read, never written.
        let game = unsafe { delta.read_unaligned() };
        let Some(local) = Boundary::run(Some(game), || LookAdapter::with(Some(game), |adapter| adapter.real(game)))
        else {
            return;
        };
        // SAFETY: the game's player and a local copy of the delta that lives for the call.
        unsafe { original(this, &local) };
    }

    unsafe extern "C" fn extract_raw_hid_input(
        abilities: u64,
        move_input: u64,
        actor_flags: u64,
        raw: *mut c_void,
        sneaking: u64,
        in_water: u64,
    ) {
        // SAFETY: the hook on `extractRawHIDInput` is installed, so its slot holds an `ExtractRawHidInputFn`.
        let original = unsafe { Self::original::<ExtractRawHidInputFn>(&EXTRACT_RAW_HID_INPUT) };
        // SAFETY: the game's own registers and slots.
        unsafe { original(abilities, move_input, actor_flags, raw, sneaking, in_water) };
        if raw.is_null() {
            return;
        }
        Boundary::run((), || MoveAdapter::with((), |adapter| adapter.real(raw.expose_provenance())));
    }

    unsafe extern "C" fn refresh(this: *mut c_void) {
        let skip = Boundary::run(false, || GamepadAdapter::with(false, |adapter| adapter.real()));
        if skip {
            return;
        }
        // SAFETY: the hook on `refresh` is installed, so its slot holds a `RefreshFn`.
        let original = unsafe { Self::original::<RefreshFn>(&REFRESH) };
        // SAFETY: the game's own handler.
        unsafe { original(this) };
    }
}
