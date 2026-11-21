use std::cell::Cell;
use std::ffi::c_void;
use std::sync::atomic::{AtomicPtr, Ordering};

use crate::{Boundary, LevelRendererAdapter, SessionHook};

/// `float LevelRendererPlayer::getFov(float a, bool enableVariableFOV) const`: rcx the renderer, xmm1 `a`,
/// r8 the bool, passed on as the whole register; the result in xmm0.
type GetFovFn = unsafe extern "C" fn(*mut c_void, f32, u64) -> f32;
/// `void LevelRendererPlayer::onWillChangeDimension(Player&)`: rcx the PlayerListener subobject, rdx the
/// player.
type WillChangeDimensionFn = unsafe extern "C" fn(*mut c_void, *mut c_void);

static GET_FOV: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
static WILL_CHANGE_DIMENSION: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());

/// The loader's detours on `LevelRendererPlayer::getFov` and `LevelRendererPlayer::onWillChangeDimension`.
///
/// The fov detour lets the adapter call the original: a call that asks for the game's own variable FOV
/// bool passes the game's arguments unchanged, the other passes variable FOV off. A panic in the adapter
/// gives the game the value of the original with its own arguments, which is called then if it did not
/// run yet. The dimension detour calls
/// the original once, with the arguments it received, and then runs the adapter inside `Boundary::run`.
pub struct LevelRendererDetours;

impl LevelRendererDetours {
    pub const GET_FOV: &'static str = "LevelRendererPlayer.getFov";
    pub const WILL_CHANGE_DIMENSION: &'static str = "LevelRendererPlayer.onWillChangeDimension";

    /// The two hooks, in install order.
    pub fn hooks() -> [SessionHook; 2] {
        [
            SessionHook {
                entry: Self::GET_FOV,
                vtable: None,
                detour: Self::get_fov as GetFovFn as *const c_void,
                original: &GET_FOV,
            },
            SessionHook {
                entry: Self::WILL_CHANGE_DIMENSION,
                vtable: None,
                detour: Self::will_change_dimension as WillChangeDimensionFn as *const c_void,
                original: &WILL_CHANGE_DIMENSION,
            },
        ]
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

    unsafe extern "C" fn get_fov(this: *mut c_void, a: f32, variable: u64) -> f32 {
        // SAFETY: the hook on `LevelRendererPlayer.getFov` is installed, so its slot holds a `GetFovFn`.
        let original = unsafe { Self::original::<GetFovFn>(&GET_FOV) };
        // The value of the original with the game's own arguments, once it ran.
        let base: Cell<Option<f32>> = Cell::new(None);
        let enabled = variable as u8 != 0;
        let call = |asked: bool| {
            if asked == enabled {
                // SAFETY: the game's own arguments, passed on unchanged.
                let value = unsafe { original(this, a, variable) };
                base.set(Some(value));
                value
            } else {
                // SAFETY: the game's renderer and `a`, with the variable FOV bool the adapter asked for.
                unsafe { original(this, a, u64::from(asked)) }
            }
        };
        let fov = Boundary::run(None, || LevelRendererAdapter::with(None, |adapter| Some(adapter.fov(enabled, call))));
        match fov.or(base.get()) {
            Some(fov) => fov,
            // SAFETY: the game's own arguments, passed on unchanged.
            None => unsafe { original(this, a, variable) },
        }
    }

    unsafe extern "C" fn will_change_dimension(this: *mut c_void, player: *mut c_void) {
        // SAFETY: the hook on `LevelRendererPlayer.onWillChangeDimension` is installed, so its slot holds a
        // `WillChangeDimensionFn`.
        let original = unsafe { Self::original::<WillChangeDimensionFn>(&WILL_CHANGE_DIMENSION) };
        // SAFETY: the game's own arguments, passed on unchanged.
        unsafe { original(this, player) };
        Boundary::run((), || LevelRendererAdapter::with((), LevelRendererAdapter::will_change_dimension));
    }
}
