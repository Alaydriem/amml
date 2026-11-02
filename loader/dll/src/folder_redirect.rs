use std::os::windows::ffi::OsStrExt;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::Path;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use windows_sys::Win32::Foundation::{E_FAIL, E_OUTOFMEMORY, HANDLE, S_OK};
use windows_sys::Win32::System::Com::CoTaskMemAlloc;
use windows_sys::Win32::UI::Shell::{CSIDL_APPDATA, FOLDERID_RoamingAppData};
use windows_sys::core::{GUID, HRESULT, PCWSTR, PWSTR};

use crate::{FolderCall, HookError, ImportHook, ImportSlot};

type KnownFolderFn = unsafe extern "system" fn(*const GUID, u32, HANDLE, *mut PWSTR) -> HRESULT;
type FolderPathFn = unsafe extern "system" fn(HANDLE, i32, HANDLE, u32, PWSTR) -> HRESULT;
type EnvironmentFn = unsafe extern "system" fn(PCWSTR, PWSTR, u32) -> u32;

// The folder the game sees as its roaming data folder, UTF-16 without a trailing separator or NUL.
static DATA_DIR: OnceLock<Vec<u16>> = OnceLock::new();
// The functions the replacements fall back to, indexed by `FolderCall::index`. Each is stored before
// its import entries are rewritten and read after, so a replacement never sees a zero.
static ORIGINALS: [AtomicUsize; FolderCall::COUNT] = [const { AtomicUsize::new(0) }; FolderCall::COUNT];
// Whether the first redirected call of a kind has been logged.
static LOGGED: [AtomicBool; FolderCall::COUNT] = [const { AtomicBool::new(false) }; FolderCall::COUNT];

/// Makes the game see a chosen folder as its roaming data folder (`%APPDATA%`), by rewriting the
/// game module's imports of the Windows functions that return it.
///
/// Other folders and other environment variables go to the original functions unchanged. The state
/// is process-global, so a process holds one redirect.
pub struct FolderRedirect {
    slots: Vec<ImportSlot>,
}

impl FolderRedirect {
    /// The functions whose imports are rewritten: the three routes to the roaming data folder.
    #[cfg(not(feature = "folder-trace"))]
    pub const FUNCTIONS: &'static [&'static str] =
        &["SHGetKnownFolderPath", "SHGetFolderPathW", "GetEnvironmentVariableW"];

    /// The functions whose imports are rewritten, with the routes only traced added.
    #[cfg(feature = "folder-trace")]
    pub const FUNCTIONS: &'static [&'static str] = &[
        "SHGetKnownFolderPath",
        "SHGetFolderPathW",
        "GetEnvironmentVariableW",
        "GetTempPathW",
        "GetTempPathA",
        "GetEnvironmentVariableA",
        "getenv",
        "getenv_s",
        "_dupenv_s",
        "_wdupenv_s",
    ];

    const MAX_PATH: usize = 260;
    const APPDATA: &'static [u8] = b"APPDATA";
    const CSIDL_MASK: i32 = 0xFF;

    /// Redirects the roaming data folder of the module at `module_base` to `data_dir`, creating the
    /// folder when it is missing. A relative `data_dir` is made absolute first.
    ///
    /// A module that imports none of [`FolderRedirect::FUNCTIONS`] is refused, because the game
    /// would keep using the real folder. A failed install leaves the process unable to install
    /// again; the host does not retry.
    ///
    /// # Safety
    /// `module_base` is the base of a PE image mapped in this process that stays mapped for the
    /// life of the process.
    pub unsafe fn install(module_base: usize, data_dir: &Path) -> Result<Self, HookError> {
        let absolute = std::path::absolute(data_dir).map_err(HookError::DataDir)?;
        std::fs::create_dir_all(&absolute).map_err(HookError::DataDir)?;
        let mut wide: Vec<u16> = absolute.as_os_str().encode_wide().collect();
        while wide.len() > 3 && wide.last().is_some_and(|unit| *unit == u16::from(b'\\')) {
            wide.pop();
        }
        DATA_DIR.set(wide).map_err(|_| HookError::InvalidArgument("a folder redirect is already installed"))?;
        let mut redirect = Self { slots: Vec::new() };
        for function in Self::FUNCTIONS {
            let (call, replacement) = Self::replacement(function);
            // SAFETY: forwarded from the caller.
            let found = match unsafe { ImportHook::find(module_base, function) } {
                Ok(found) => found,
                Err(error) => {
                    redirect.restore();
                    return Err(error);
                }
            };
            let Some(first) = found.first() else {
                continue;
            };
            // Every entry of one function resolves to one implementation, so the first entry's
            // original serves the replacement; each slot restores its own value.
            ORIGINALS[call.index()].store(first.original(), Ordering::SeqCst);
            // SAFETY: the slots were just found in the module the caller vouches for, and the
            // replacement has the signature of the function it replaces.
            if let Err(error) = unsafe { ImportHook::apply(&found, replacement) } {
                redirect.restore();
                return Err(error);
            }
            redirect.slots.extend(found);
        }
        if redirect.slots.is_empty() {
            return Err(HookError::ImportTable("the module imports none of the folder functions"));
        }
        Ok(redirect)
    }

    /// The import entries that were rewritten.
    pub fn slots(&self) -> &[ImportSlot] {
        &self.slots
    }

    /// Puts the original imports back. The replacements stay valid for calls already in flight.
    pub fn restore(&self) {
        for slot in &self.slots {
            // A failed restore leaves the replacement in place, which still answers correctly.
            let _ = slot.restore();
        }
    }

    fn replacement(function: &str) -> (FolderCall, usize) {
        match function {
            "SHGetKnownFolderPath" => (FolderCall::KnownFolder, Self::known_folder as *const () as usize),
            "SHGetFolderPathW" => (FolderCall::FolderPath, Self::folder_path as *const () as usize),
            "GetEnvironmentVariableW" => (FolderCall::Environment, Self::environment as *const () as usize),
            other => Self::traced(other),
        }
    }

    // The trace build also routes the functions that are only logged.
    #[cfg(feature = "folder-trace")]
    fn traced(function: &str) -> (FolderCall, usize) {
        crate::trace_hooks::TraceHooks::replacement(function)
            .unwrap_or((FolderCall::Environment, Self::environment as *const () as usize))
    }

    #[cfg(not(feature = "folder-trace"))]
    fn traced(_function: &str) -> (FolderCall, usize) {
        (FolderCall::Environment, Self::environment as *const () as usize)
    }

    pub(crate) fn original<F: Copy>(call: FolderCall) -> Option<F> {
        let address = ORIGINALS[call.index()].load(Ordering::SeqCst);
        if address == 0 || size_of::<F>() != size_of::<usize>() {
            return None;
        }
        // SAFETY: `F` is a function pointer type of pointer size, and the address is the import
        // entry's value before it was rewritten: the Windows function `call` stands for.
        Some(unsafe { std::mem::transmute_copy::<usize, F>(&address) })
    }

    fn log_once(call: FolderCall, path: &[u16]) {
        if LOGGED[call.index()].swap(true, Ordering::Relaxed) {
            return;
        }
        curia::info!("folder redirected", { call: call.function(), path: String::from_utf16_lossy(path) });
    }

    extern "system" fn known_folder(rfid: *const GUID, flags: u32, token: HANDLE, out: *mut PWSTR) -> HRESULT {
        let handled = catch_unwind(AssertUnwindSafe(|| Self::try_known_folder(rfid, out)));
        if let Ok(Some(result)) = handled {
            return result;
        }
        match Self::original::<KnownFolderFn>(FolderCall::KnownFolder) {
            // SAFETY: the original takes the arguments the caller passed to this replacement.
            Some(original) => unsafe { original(rfid, flags, token, out) },
            None => E_FAIL,
        }
    }

    fn try_known_folder(rfid: *const GUID, out: *mut PWSTR) -> Option<HRESULT> {
        let data_dir = DATA_DIR.get()?;
        if rfid.is_null() || out.is_null() {
            return None;
        }
        // SAFETY: the caller passes a valid GUID pointer, as `SHGetKnownFolderPath` requires; it is non-null here.
        let requested = unsafe { &*rfid };
        #[cfg(feature = "folder-trace")]
        crate::folder_trace::FolderTrace::known_folder(requested);
        if !Self::same_guid(requested, &FOLDERID_RoamingAppData) {
            return None;
        }
        // SAFETY: `CoTaskMemAlloc` has no preconditions; the caller frees the block with `CoTaskMemFree`.
        let block = unsafe { CoTaskMemAlloc((data_dir.len() + 1) * size_of::<u16>()) }.cast::<u16>();
        if block.is_null() {
            // SAFETY: `out` is a valid out pointer; the API contract is that it is NULL on failure.
            unsafe { *out = std::ptr::null_mut() };
            return Some(E_OUTOFMEMORY);
        }
        // SAFETY: `block` holds `len + 1` units, and `out` is a valid out pointer, as the API requires.
        unsafe {
            std::ptr::copy_nonoverlapping(data_dir.as_ptr(), block, data_dir.len());
            *block.add(data_dir.len()) = 0;
            *out = block;
        }
        Self::log_once(FolderCall::KnownFolder, data_dir);
        Some(S_OK)
    }

    fn same_guid(left: &GUID, right: &GUID) -> bool {
        left.data1 == right.data1 && left.data2 == right.data2 && left.data3 == right.data3 && left.data4 == right.data4
    }

    extern "system" fn folder_path(hwnd: HANDLE, csidl: i32, token: HANDLE, flags: u32, path: PWSTR) -> HRESULT {
        let handled = catch_unwind(AssertUnwindSafe(|| Self::try_folder_path(csidl, path)));
        if let Ok(Some(result)) = handled {
            return result;
        }
        match Self::original::<FolderPathFn>(FolderCall::FolderPath) {
            // SAFETY: the original takes the arguments the caller passed to this replacement.
            Some(original) => unsafe { original(hwnd, csidl, token, flags, path) },
            None => E_FAIL,
        }
    }

    fn try_folder_path(csidl: i32, path: PWSTR) -> Option<HRESULT> {
        #[cfg(feature = "folder-trace")]
        crate::folder_trace::FolderTrace::folder_path(csidl);
        let data_dir = DATA_DIR.get()?;
        if path.is_null() || (csidl & Self::CSIDL_MASK) != CSIDL_APPDATA as i32 {
            return None;
        }
        if data_dir.len() + 1 > Self::MAX_PATH {
            return Some(E_FAIL);
        }
        // SAFETY: `SHGetFolderPathW` requires a buffer of `MAX_PATH` units, and the path with its NUL fits.
        unsafe {
            std::ptr::copy_nonoverlapping(data_dir.as_ptr(), path, data_dir.len());
            *path.add(data_dir.len()) = 0;
        }
        Self::log_once(FolderCall::FolderPath, data_dir);
        Some(S_OK)
    }

    extern "system" fn environment(name: PCWSTR, buffer: PWSTR, size: u32) -> u32 {
        let handled = catch_unwind(AssertUnwindSafe(|| Self::try_environment(name, buffer, size)));
        if let Ok(Some(result)) = handled {
            return result;
        }
        match Self::original::<EnvironmentFn>(FolderCall::Environment) {
            // SAFETY: the original takes the arguments the caller passed to this replacement.
            Some(original) => unsafe { original(name, buffer, size) },
            None => 0,
        }
    }

    fn try_environment(name: PCWSTR, buffer: PWSTR, size: u32) -> Option<u32> {
        #[cfg(feature = "folder-trace")]
        crate::folder_trace::FolderTrace::wide(FolderCall::Environment, name);
        let data_dir = DATA_DIR.get()?;
        if !Self::is_appdata(name) {
            return None;
        }
        let length = data_dir.len();
        // The documented contract: a buffer that holds the value and its NUL gets the value and the
        // length without the NUL; a smaller one is untouched and gets the length with the NUL.
        if buffer.is_null() || (size as usize) < length + 1 {
            return Some((length + 1) as u32);
        }
        // SAFETY: `buffer` holds `size` units and `length + 1 <= size`.
        unsafe {
            std::ptr::copy_nonoverlapping(data_dir.as_ptr(), buffer, length);
            *buffer.add(length) = 0;
        }
        Self::log_once(FolderCall::Environment, data_dir);
        Some(length as u32)
    }

    fn is_appdata(name: PCWSTR) -> bool {
        if name.is_null() {
            return false;
        }
        for (position, expected) in Self::APPDATA.iter().enumerate() {
            // SAFETY: `name` is NUL-terminated; the walk stops at the first unit that differs, and a
            // NUL differs from every letter, so it never reads past the terminator.
            let unit = unsafe { *name.add(position) };
            if u8::try_from(unit).map(|byte| byte.to_ascii_uppercase()) != Ok(*expected) {
                return false;
            }
        }
        // SAFETY: all seven units matched, so the next one is still inside the string.
        unsafe { *name.add(Self::APPDATA.len()) == 0 }
    }
}
