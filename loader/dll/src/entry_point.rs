use std::ffi::c_void;
use std::sync::OnceLock;

use retour::RawDetour;
use windows_sys::Win32::System::Diagnostics::Debug::OutputDebugStringW;
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::w;

use crate::{HostProcess, StandaloneHost, StartupGate};

static INSTALLED: OnceLock<EntryPoint> = OnceLock::new();
static GATE: StartupGate = StartupGate::new();

/// The one-shot detour on the exe entry point that starts the standalone host.
///
/// `DllMain` runs under the OS loader lock, where loading mods, creating files or starting threads
/// is unsafe. It only installs this detour. When the game's main thread reaches its entry point the
/// loader lock is released: the detour runs the standalone start-up once, removes itself and calls
/// the original entry point.
///
/// This detour is not part of `HookEngine`: it does not count against `max_hooks`, and it is
/// disabled after start-up. It lives in a `static` and is never dropped.
pub struct EntryPoint {
    detour: RawDetour,
    target: usize,
}

/// The exe entry point. Windows passes the PEB address, which the CRT entry ignores; it is forwarded.
type EntryFn = extern "system" fn(*mut c_void) -> u32;

impl EntryPoint {
    const HEADER_BYTES: usize = 0x1000;
    const DOS_MAGIC: &'static [u8] = b"MZ";
    const PE_MAGIC: &'static [u8] = b"PE\0\0";
    const LFANEW_OFFSET: usize = 0x3C;
    const OPTIONAL_HEADER_OFFSET: usize = 24;
    const PE32_PLUS_MAGIC: u16 = 0x20B;
    const ENTRY_POINT_OFFSET: usize = 16;

    /// `AddressOfEntryPoint` of a 64-bit PE image, read from its headers (the first bytes of the
    /// file or of the mapped image). `None` for headers that are not PE32+, are cut short, or name
    /// no entry point.
    pub fn rva_from_headers(headers: &[u8]) -> Option<u32> {
        if headers.get(..2)? != Self::DOS_MAGIC {
            return None;
        }
        let nt = usize::try_from(Self::read_u32(headers, Self::LFANEW_OFFSET)?).ok()?;
        if headers.get(nt..nt.checked_add(4)?)? != Self::PE_MAGIC {
            return None;
        }
        let optional = nt.checked_add(Self::OPTIONAL_HEADER_OFFSET)?;
        let magic = u16::from_le_bytes(headers.get(optional..optional.checked_add(2)?)?.try_into().ok()?);
        if magic != Self::PE32_PLUS_MAGIC {
            return None;
        }
        let rva = Self::read_u32(headers, optional.checked_add(Self::ENTRY_POINT_OFFSET)?)?;
        (rva != 0).then_some(rva)
    }

    /// Whether LeviLamina hosts this process. Its `PreLoader.dll` loads before `amml.dll`.
    pub fn is_levilamina_host() -> bool {
        // SAFETY: the argument is a NUL-terminated UTF-16 literal. The call takes no reference on
        // the module and is allowed under the loader lock.
        let handle = unsafe { GetModuleHandleW(w!("PreLoader.dll")) };
        !handle.is_null()
    }

    /// Called from `DllMain(DLL_PROCESS_ATTACH)` for a static load. Outside the LeviLamina host,
    /// detours the exe entry point to start the standalone host. Does nothing else: no file, no log,
    /// no thread. When the entry point cannot be found or detoured, the game starts without AMML and
    /// the reason goes to `OutputDebugStringW`, which is safe under the loader lock.
    pub fn install_if_standalone() {
        if Self::is_levilamina_host() {
            return;
        }
        let base = HostProcess::module_base();
        if base == 0 {
            Self::debug_line(w!("amml: no exe module; the standalone host is off"));
            return;
        }
        // SAFETY: `base` is the base of the running exe's mapped image, whose first page holds its
        // headers and is always readable.
        let headers = unsafe { std::slice::from_raw_parts(base as usize as *const u8, Self::HEADER_BYTES) };
        let Some(rva) = Self::rva_from_headers(headers) else {
            Self::debug_line(w!("amml: the exe has no readable entry point; the standalone host is off"));
            return;
        };
        let target = base as usize + rva as usize;
        // SAFETY: `target` is the exe's entry point from its own mapped headers. The main thread has
        // not reached it yet: DLLs in the import table attach before the entry point runs.
        if unsafe { Self::install(target) }.is_err() {
            Self::debug_line(w!("amml: the entry point detour was not installed; the standalone host is off"));
        }
    }

    fn debug_line(text: *const u16) {
        // SAFETY: `text` is a NUL-terminated UTF-16 literal from `w!`; the call only writes to the
        // debugger, if one is attached.
        unsafe { OutputDebugStringW(text) };
    }

    /// # Safety
    /// `target` must be the entry point of the running exe, not yet running on any thread.
    unsafe fn install(target: usize) -> Result<(), retour::Error> {
        let detour_fn: EntryFn = Self::detour;
        // SAFETY: the caller guarantees `target`; `detour_fn` has the entry point's signature.
        let detour = unsafe { RawDetour::new(target as *const (), detour_fn as *const ())? };
        if INSTALLED.set(Self { detour, target }).is_err() {
            return Ok(());
        }
        match INSTALLED.get() {
            // SAFETY: the detour was validated by RawDetour::new above, and the entry point is not running.
            Some(entry) => unsafe { entry.detour.enable() },
            None => Ok(()),
        }
    }

    extern "system" fn detour(peb: *mut c_void) -> u32 {
        GATE.pass(|| {
            if !StandaloneHost::run() {
                curia::error!("the standalone host did not start; the game continues without AMML mods");
            }
        });
        Self::call_game(peb)
    }

    // Removes the detour and calls the original entry point. When the detour cannot be removed, the
    // trampoline is called instead, so the game never re-enters the detour.
    fn call_game(peb: *mut c_void) -> u32 {
        let Some(entry) = INSTALLED.get() else {
            // `install` stores the detour before it enables it, so the detour never runs without it.
            debug_assert!(false, "the entry point detour ran before it was stored");
            return 1;
        };
        // SAFETY: the main thread is inside the detour, not in the patched prologue, so restoring
        // the original bytes races with no running code.
        if let Err(error) = unsafe { entry.detour.disable() } {
            curia::warn!("the entry point detour was not removed", { error: error.to_string() });
        }
        let address = if entry.detour.is_enabled() {
            std::ptr::from_ref(entry.detour.trampoline()) as usize
        } else {
            entry.target
        };
        // SAFETY: `address` is the exe entry point with its original bytes, or the trampoline that
        // runs them; both have the entry point's signature.
        let original = unsafe { std::mem::transmute::<usize, EntryFn>(address) };
        original(peb)
    }

    fn read_u32(bytes: &[u8], offset: usize) -> Option<u32> {
        let slice = bytes.get(offset..offset.checked_add(4)?)?;
        Some(u32::from_le_bytes(slice.try_into().ok()?))
    }
}
