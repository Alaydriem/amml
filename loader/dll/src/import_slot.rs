use std::sync::atomic::{AtomicUsize, Ordering};

use windows_sys::Win32::System::Memory::{
    PAGE_EXECUTE_READWRITE, PAGE_PROTECTION_FLAGS, PAGE_READWRITE, VirtualProtect,
};

use crate::HookError;

/// One import address table entry found by [`ImportHook::find`](crate::ImportHook::find).
///
/// A slot exists only for an entry of a module that the caller of `find` promised stays mapped,
/// so writing to it is sound.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ImportSlot {
    module_base: usize,
    slot: *mut usize,
    original: usize,
    function: &'static str,
}

// SAFETY: the entry is plain memory in a module's import table that stays mapped; the slot is an
// address to write an aligned `usize` to, which any thread may do.
unsafe impl Send for ImportSlot {}
// SAFETY: as for `Send`; every write goes through an atomic store.
unsafe impl Sync for ImportSlot {}

impl ImportSlot {
    // Protection bits that allow execution: PAGE_EXECUTE, _READ, _READWRITE and _WRITECOPY.
    const EXECUTE_BITS: PAGE_PROTECTION_FLAGS = 0xF0;

    pub(crate) fn new(module_base: usize, slot: *mut usize, original: usize, function: &'static str) -> Self {
        Self { module_base, slot, original, function }
    }

    /// Base address of the module whose table holds the entry.
    pub fn module_base(&self) -> usize {
        self.module_base
    }

    /// The address of the table entry.
    pub fn address(&self) -> usize {
        self.slot as usize
    }

    /// The address the entry held before it was patched.
    pub fn original(&self) -> usize {
        self.original
    }

    /// The imported function's name.
    pub fn function(&self) -> &'static str {
        self.function
    }

    /// Puts the original address back.
    pub fn restore(&self) -> Result<(), HookError> {
        self.write(self.original)
    }

    /// Writes `value` into the entry, making its page writable for the write.
    pub(crate) fn write(&self, value: usize) -> Result<(), HookError> {
        let mut previous: PAGE_PROTECTION_FLAGS = 0;
        // SAFETY: `slot` is an entry of a loaded module's import table, found by `ImportHook::find`
        // under its promise that the module stays mapped; the call changes page protection only.
        let probed = unsafe { VirtualProtect(self.slot.cast(), size_of::<usize>(), PAGE_READWRITE, &mut previous) };
        if probed == 0 {
            return Err(HookError::Protect(std::io::Error::last_os_error()));
        }
        // A page that is also executable keeps its execute right while it is writable.
        let writable = if previous & Self::EXECUTE_BITS != 0 { PAGE_EXECUTE_READWRITE } else { PAGE_READWRITE };
        if writable != PAGE_READWRITE {
            let mut ignored: PAGE_PROTECTION_FLAGS = 0;
            // SAFETY: the same page and the same promise as above.
            let changed = unsafe { VirtualProtect(self.slot.cast(), size_of::<usize>(), writable, &mut ignored) };
            if changed == 0 {
                return Err(HookError::Protect(std::io::Error::last_os_error()));
            }
        }
        // SAFETY: the page is writable after the calls above, and `slot` is aligned and valid for a
        // `usize`. The atomic store lets another thread calling through the entry see either the
        // old or the new address, never a torn one.
        unsafe { AtomicUsize::from_ptr(self.slot) }.store(value, Ordering::SeqCst);
        let mut ignored: PAGE_PROTECTION_FLAGS = 0;
        // SAFETY: the same page as above; `previous` is the protection the first call reported.
        let restored = unsafe { VirtualProtect(self.slot.cast(), size_of::<usize>(), previous, &mut ignored) };
        if restored == 0 {
            return Err(HookError::Protect(std::io::Error::last_os_error()));
        }
        Ok(())
    }
}
