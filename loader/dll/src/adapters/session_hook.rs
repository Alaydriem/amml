use std::ffi::c_void;
use std::sync::atomic::AtomicPtr;

use crate::LoaderState;

/// One game function a loader adapter detours: its entry, the vtable global that holds it when the
/// entry is a slot, the loader's detour and the slot that receives the original.
pub struct SessionHook {
    pub entry: &'static str,
    /// The vtable global of a `vslot` entry; `None` for a `function` entry.
    pub vtable: Option<&'static str>,
    pub detour: *const c_void,
    pub original: &'static AtomicPtr<c_void>,
}

impl SessionHook {
    /// Installs this detour as a loader hook at `priority`; false, with a log line, when the target is
    /// unknown or the hook is refused.
    pub(crate) fn install(&self, state: &'static LoaderState, priority: i32) -> bool {
        let Some(target) = self.target(state) else {
            curia::warn!("a loader hook has no target", { entry: self.entry });
            return false;
        };
        // SAFETY: `target` is the start of the game function of `entry` (a function entry, or the slot of a
        // vtable global read from the image), `detour` has its signature, and `original` is a static
        // `AtomicPtr` that is read only through the atomic.
        let installed = unsafe {
            state.install_loader_hook(target, self.detour, priority, self.original.as_ptr().cast::<*const c_void>())
        };
        match installed {
            Ok(_) => true,
            Err(status) => {
                curia::warn!("a loader hook was refused", { entry: self.entry, status: status.0 });
                false
            }
        }
    }

    /// The address of the function behind this hook: the entry for a function, the function in the vtable
    /// slot for a `vslot`.
    pub(crate) fn target(&self, state: &LoaderState) -> Option<*const c_void> {
        let value = state.resolve(self.entry).ok()?;
        let address = match self.vtable {
            None => value,
            Some(vtable) => {
                let slot = state.resolve(vtable).ok()?.checked_add(value.checked_mul(8)?)?;
                let slot = usize::try_from(slot).ok()? as *const usize;
                // SAFETY: `slot` is inside the vtable global of the game image, which is mapped and relocated
                // for the life of the process, and 8-aligned like every vtable.
                unsafe { slot.read() as u64 }
            }
        };
        let address = usize::try_from(address).ok().filter(|address| *address != 0)?;
        Some(std::ptr::with_exposed_provenance::<c_void>(address))
    }
}
