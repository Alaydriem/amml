use std::ffi::c_void;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicPtr, AtomicU32, Ordering};

use crate::{BlockEntityAdapter, Boundary, LoaderState, SessionHook};

/// `void LevelRendererCamera::renderBlockEntities(BaseActorRenderContext&, bool renderAlphaLayer)`, the body every
/// camera shares: rcx the camera, rdx the context, r8b the alpha flag.
type RenderBlockEntitiesFn = unsafe extern "C" fn(usize, usize, u8);

static ORIGINAL: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
// What the detour has work for besides the game's own call. Zero: it calls the original and nothing else.
static WANTED: AtomicU32 = AtomicU32::new(0);
static INSTALLED: OnceLock<bool> = OnceLock::new();

/// The loader's detour behind `BlockEntityPass` (on `LevelRendererCamera.renderBlockEntitiesBody`) and the flags word it
/// reads.
///
/// With no flag set the detour calls the original and does nothing else: one relaxed load, no lock, no game read.
/// `LISTENER` is set while a mod listens to `BlockEntityPass`; the detour then raises the event after the game's opaque
/// pass (alpha flag 0), inside `Boundary::run`, so a panic never reaches the game.
pub struct BlockEntityDetours;

impl BlockEntityDetours {
    pub const ENTRY: &'static str = "LevelRendererCamera.renderBlockEntitiesBody";
    /// A mod listens to `BlockEntityPass`.
    pub const LISTENER: u32 = 1;
    // Loader hooks run before every mod hook on the same function.
    const PRIORITY: i32 = i32::MAX;

    pub fn hook() -> SessionHook {
        SessionHook {
            entry: Self::ENTRY,
            vtable: None,
            detour: Self::detour as RenderBlockEntitiesFn as *const c_void,
            original: &ORIGINAL,
        }
    }

    /// Installs the detour once. Every later call gives the result of the first.
    pub(crate) fn install(state: &'static LoaderState) -> bool {
        *INSTALLED.get_or_init(|| Self::hook().install(state, Self::PRIORITY))
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

    unsafe extern "C" fn detour(this: usize, context: usize, alpha: u8) {
        let address = ORIGINAL.load(Ordering::Acquire);
        // SAFETY: the engine set the slot before it moved the thunk to this detour; the hooked function has the
        // signature `RenderBlockEntitiesFn`.
        let original = unsafe { std::mem::transmute::<*mut c_void, RenderBlockEntitiesFn>(address) };
        // SAFETY: the game's own arguments, passed on unchanged.
        unsafe { original(this, context, alpha) };
        if Self::flags() & Self::LISTENER != 0 && alpha == 0 {
            Boundary::run((), || BlockEntityAdapter::with((), |adapter| adapter.pass(this, context)));
        }
    }
}
