use std::ffi::c_void;

use retour::RawDetour;

use crate::{HookError, hook_thunk::HookThunk};

/// The one detour of a target address, with the thunk it jumps to.
///
/// `enable` patches the target's prologue and is not atomic against a thread that is running that
/// prologue at the same moment. The first hook on a target must therefore be installed before the
/// game runs that code (in the loader's load/enable phase); a later first hook accepts that risk.
/// After `enable` the detour is never patched again: the chain changes by moving the thunk, and
/// an empty chain points the thunk at the trampoline. The detour is declared before the thunk so
/// it is removed first on drop.
pub struct TargetHook {
    detour: RawDetour,
    thunk: HookThunk,
}

// SAFETY: a RawDetour owns only its trampoline and the patched target, both process-global and not
// tied to the creating thread, and HookThunk is an executable page with an atomic slot. The engine
// that owns these takes `&mut self` for every change.
unsafe impl Send for TargetHook {}

impl TargetHook {
    /// Hooks `target` to jump to `first`, but does not enable the detour yet.
    ///
    /// # Safety
    /// `target` must be the start of a function in executable memory that can be detoured, and `first`
    /// a function with the same signature.
    pub unsafe fn new(target: *const c_void, first: *const c_void) -> Result<Self, HookError> {
        let thunk = HookThunk::new(first)?;
        // SAFETY: the caller guarantees `target`; the thunk entry is valid executable code.
        let detour = unsafe { RawDetour::new(target.cast(), thunk.entry().cast())? };
        Ok(Self { detour, thunk })
    }

    /// Installs the detour. Call once, after the `original` slots are written.
    pub fn enable(&self) -> Result<(), HookError> {
        // SAFETY: the target was validated by RawDetour::new in `new`.
        unsafe { self.detour.enable()? };
        Ok(())
    }

    /// The original function: a copy of the displaced prologue that continues into the target.
    pub fn trampoline(&self) -> *const c_void {
        std::ptr::from_ref(self.detour.trampoline()).cast()
    }

    pub fn point_at(&self, address: *const c_void) {
        self.thunk.point_at(address);
    }
}
