use std::ffi::{CStr, c_char};

use windows_sys::Win32::System::Diagnostics::Debug::{
    IMAGE_DIRECTORY_ENTRY_IMPORT, IMAGE_NT_HEADERS64, IMAGE_NT_OPTIONAL_HDR64_MAGIC,
};
use windows_sys::Win32::System::SystemServices::{
    IMAGE_DOS_HEADER, IMAGE_DOS_SIGNATURE, IMAGE_IMPORT_DESCRIPTOR, IMAGE_ORDINAL_FLAG64,
};

use crate::{HookError, ImportSlot};

/// Rewrites import address table entries of a loaded module.
pub struct ImportHook;

impl ImportHook {
    const NT_SIGNATURE: u32 = 0x0000_4550;
    // A bound on the walks, so a corrupt table cannot loop forever.
    const MAX_DESCRIPTORS: usize = 4096;
    const MAX_THUNKS: usize = 65536;

    /// Points every by-name import of `function` in the module at `module_base` to `replacement`,
    /// whatever DLL name the import goes through (API-set names included). Returns one
    /// [`ImportSlot`] per rewritten entry; a module without such an import gives an empty list.
    ///
    /// On an error the entries already rewritten are put back.
    ///
    /// # Safety
    /// `module_base` is the base of a PE image mapped in this process that stays mapped while any
    /// returned slot is used, and `replacement` is a function with the signature of `function`.
    pub unsafe fn patch(
        module_base: usize,
        function: &'static str,
        replacement: usize,
    ) -> Result<Vec<ImportSlot>, HookError> {
        // SAFETY: forwarded from the caller.
        let found = unsafe { Self::find(module_base, function) }?;
        // SAFETY: the slots were just found in the module the caller vouches for.
        unsafe { Self::apply(&found, replacement) }?;
        Ok(found)
    }

    /// Writes `replacement` into every entry of `slots`. On an error the entries already written
    /// are put back.
    ///
    /// # Safety
    /// The modules that hold the slots are still mapped, and `replacement` is a function with the
    /// signature of the imported function.
    pub unsafe fn apply(slots: &[ImportSlot], replacement: usize) -> Result<(), HookError> {
        for (position, slot) in slots.iter().enumerate() {
            if let Err(error) = slot.write(replacement) {
                for done in &slots[..position] {
                    // The first error is the one worth reporting.
                    let _ = done.restore();
                }
                return Err(error);
            }
        }
        Ok(())
    }

    /// The by-name import entries of `function` in the module at `module_base`, without changing
    /// them.
    ///
    /// # Safety
    /// `module_base` is the base of a PE image mapped in this process that stays mapped while any
    /// returned slot is used.
    pub unsafe fn find(module_base: usize, function: &'static str) -> Result<Vec<ImportSlot>, HookError> {
        let base = module_base as *const u8;
        // SAFETY: the caller of `find` guarantees `module_base` is a mapped image, so its DOS header
        // is readable.
        let dos = unsafe { base.cast::<IMAGE_DOS_HEADER>().read_unaligned() };
        if dos.e_magic != IMAGE_DOS_SIGNATURE {
            return Err(HookError::ImportTable("not a module image"));
        }
        let nt_offset = usize::try_from(dos.e_lfanew).map_err(|_| HookError::ImportTable("bad NT header offset"))?;
        // SAFETY: `e_lfanew` of a mapped module points at its NT headers inside the image.
        let nt = unsafe { base.add(nt_offset).cast::<IMAGE_NT_HEADERS64>().read_unaligned() };
        if nt.Signature != Self::NT_SIGNATURE || nt.OptionalHeader.Magic != IMAGE_NT_OPTIONAL_HDR64_MAGIC {
            return Err(HookError::ImportTable("not a 64-bit module image"));
        }
        let directory = nt.OptionalHeader.DataDirectory[usize::from(IMAGE_DIRECTORY_ENTRY_IMPORT)];
        let mut slots = Vec::new();
        if directory.VirtualAddress == 0 {
            return Ok(slots);
        }
        for index in 0..Self::MAX_DESCRIPTORS {
            // SAFETY: descriptors are laid out back to back from the directory start and end with an
            // all-zero entry; the walk stops there or at `MAX_DESCRIPTORS`.
            let descriptor = unsafe {
                base.add(directory.VirtualAddress as usize)
                    .cast::<IMAGE_IMPORT_DESCRIPTOR>()
                    .add(index)
                    .read_unaligned()
            };
            // SAFETY: reading the `OriginalFirstThunk` view of a plain-data union of two `u32`.
            let names_rva = unsafe { descriptor.Anonymous.OriginalFirstThunk };
            if names_rva == 0 && descriptor.FirstThunk == 0 {
                break;
            }
            if names_rva == 0 || descriptor.FirstThunk == 0 {
                continue;
            }
            // SAFETY: the descriptor belongs to the image the caller of `find` vouches for.
            unsafe { Self::scan_thunks(base, names_rva, descriptor.FirstThunk, function, &mut slots) };
        }
        Ok(slots)
    }

    // SAFETY (callers): `base` is a mapped image and the two RVAs come from one of its import descriptors.
    unsafe fn scan_thunks(
        base: *const u8,
        names_rva: u32,
        table_rva: u32,
        function: &'static str,
        slots: &mut Vec<ImportSlot>,
    ) {
        for position in 0..Self::MAX_THUNKS {
            // SAFETY: the name table is a zero-terminated array of `u64` inside the image; the walk
            // stops at the terminator or at `MAX_THUNKS`.
            let thunk = unsafe { base.add(names_rva as usize).cast::<u64>().add(position).read_unaligned() };
            if thunk == 0 {
                break;
            }
            if thunk & IMAGE_ORDINAL_FLAG64 != 0 {
                continue;
            }
            // SAFETY: a by-name thunk is the RVA of an `IMAGE_IMPORT_BY_NAME`: a `u16` hint, then a
            // NUL-terminated name, all inside the image.
            let name = unsafe { CStr::from_ptr(base.add(thunk as usize + 2).cast::<c_char>()) };
            if name.to_bytes() != function.as_bytes() {
                continue;
            }
            // SAFETY: the address table has the same length as the name table, so entry `position` exists.
            let slot = unsafe { base.add(table_rva as usize).cast::<usize>().add(position) }.cast_mut();
            // SAFETY: `slot` is that table entry, aligned for a `usize`.
            let original = unsafe { slot.read() };
            slots.push(ImportSlot::new(base as usize, slot, original, function));
        }
    }
}
