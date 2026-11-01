use std::ffi::c_void;
use std::sync::OnceLock;

use windows_sys::Win32::System::LibraryLoader::{GetModuleHandleW, GetProcAddress};

/// `void* pl_resolve_symbol_silent_n(const char* name, size_t len)`
type ResolveFn = unsafe extern "C" fn(*const u8, usize) -> *mut c_void;

static RESOLVER: OnceLock<Option<ResolveFn>> = OnceLock::new();

/// LeviLamina's symbol lookup, exported by `PreLoader.dll`.
///
/// `SymbolLookup` is a plain function pointer, so the resolved export lives in a static that
/// `LlSymbols::lookup` reads. When `PreLoader.dll` or the export is missing the lookup finds nothing.
pub struct LlSymbols;

impl LlSymbols {
    const MODULE: &'static str = "PreLoader.dll";
    const EXPORT: &'static [u8] = b"pl_resolve_symbol_silent_n\0";

    /// Finds the export once; a miss is logged once. Returns whether lookups can succeed.
    pub fn install() -> bool {
        RESOLVER.get_or_init(Self::find).is_some()
    }

    /// The absolute address of an MSVC-decorated symbol, or `None`.
    pub fn lookup(name: &str) -> Option<u64> {
        let resolve = RESOLVER.get().copied().flatten()?;
        // SAFETY: `resolve` is the PreLoader export with this signature; `name` is valid for `len` bytes.
        let address = unsafe { resolve(name.as_ptr(), name.len()) };
        if address.is_null() { None } else { Some(address as usize as u64) }
    }

    fn find() -> Option<ResolveFn> {
        let module: Vec<u16> = Self::MODULE.encode_utf16().chain(std::iter::once(0)).collect();
        // SAFETY: `module` is a NUL-terminated UTF-16 name that lives for the call.
        let handle = unsafe { GetModuleHandleW(module.as_ptr()) };
        if handle.is_null() {
            curia::warn!("PreLoader.dll is not loaded; LeviLamina symbol lookup is off");
            return None;
        }
        // SAFETY: `handle` is a loaded module and `EXPORT` is NUL-terminated.
        let Some(address) = (unsafe { GetProcAddress(handle, Self::EXPORT.as_ptr()) }) else {
            curia::warn!("PreLoader.dll has no pl_resolve_symbol_silent_n; LeviLamina symbol lookup is off");
            return None;
        };
        // SAFETY: the export has the `ResolveFn` signature.
        Some(unsafe { std::mem::transmute::<unsafe extern "system" fn() -> isize, ResolveFn>(address) })
    }
}
