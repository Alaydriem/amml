//! A DLL that asks Windows for folders through its normal imports, for the import hook and folder redirect tests.

use std::ffi::{c_char, c_void};
use std::ptr::null_mut;

use windows_sys::Win32::Foundation::MAX_PATH;
use windows_sys::Win32::Storage::FileSystem::{GetTempPathA, GetTempPathW};
use windows_sys::Win32::System::Com::CoTaskMemFree;
use windows_sys::Win32::System::Environment::GetEnvironmentVariableA;
use windows_sys::Win32::UI::Shell::{
    CSIDL_APPDATA, CSIDL_PERSONAL, FOLDERID_LocalAppData, FOLDERID_RoamingAppData, SHGetFolderPathW,
    SHGetKnownFolderPath,
};
use windows_sys::core::GUID;

// Imported through an API-set DLL name, as the game's own imports may be.
#[link(name = "api-ms-win-core-processenvironment-l1-1-0", kind = "raw-dylib")]
unsafe extern "system" {
    fn GetEnvironmentVariableW(name: *const u16, buffer: *mut u16, size: u32) -> u32;
}

unsafe extern "C" {
    fn getenv(name: *const c_char) -> *mut c_char;
    fn getenv_s(size: *mut usize, buffer: *mut c_char, capacity: usize, name: *const c_char) -> i32;
    fn _dupenv_s(buffer: *mut *mut c_char, count: *mut usize, name: *const c_char) -> i32;
    fn _wdupenv_s(buffer: *mut *mut u16, count: *mut usize, name: *const u16) -> i32;
    fn free(block: *mut c_void);
}

struct Probe;

impl Probe {
    // Copies a NUL-terminated UTF-16 string to `out` when it fits in `cap`, and returns its length.
    //
    // SAFETY (callers): `text` is NUL-terminated and `out` is valid for `cap` units.
    unsafe fn copy_out(text: *const u16, out: *mut u16, cap: u32) -> u32 {
        let mut length = 0usize;
        // SAFETY: `text` is NUL-terminated.
        while unsafe { *text.add(length) } != 0 {
            length += 1;
        }
        if length < cap as usize {
            // SAFETY: `out` holds `cap` units and `length + 1 <= cap`.
            unsafe { std::ptr::copy_nonoverlapping(text, out, length + 1) };
        }
        length as u32
    }

    // Copies a NUL-terminated narrow string to `out` when it fits in `cap`, and returns its length.
    //
    // SAFETY (callers): `text` is NUL-terminated and `out` is valid for `cap` bytes.
    unsafe fn copy_narrow(text: *const u8, out: *mut u8, cap: u32) -> u32 {
        let mut length = 0usize;
        // SAFETY: `text` is NUL-terminated.
        while unsafe { *text.add(length) } != 0 {
            length += 1;
        }
        if length < cap as usize {
            // SAFETY: `out` holds `cap` bytes and `length + 1 <= cap`.
            unsafe { std::ptr::copy_nonoverlapping(text, out, length + 1) };
        }
        length as u32
    }

    unsafe fn known_folder(id: &GUID, out: *mut u16, cap: u32) -> u32 {
        let mut path: *mut u16 = null_mut();
        // SAFETY: `id` is a valid GUID and `path` is a valid out pointer.
        let status = unsafe { SHGetKnownFolderPath(id, 0, null_mut(), &mut path) };
        if status != 0 || path.is_null() {
            return 0;
        }
        // SAFETY: `path` is a NUL-terminated string from the shell; `out` and `cap` come from the caller.
        let length = unsafe { Self::copy_out(path, out, cap) };
        // SAFETY: the shell allocated `path` with CoTaskMemAlloc.
        unsafe { CoTaskMemFree(path as *const c_void) };
        length
    }

    unsafe fn folder_path(csidl: u32, out: *mut u16) -> i32 {
        // SAFETY: the caller passes a `MAX_PATH` buffer.
        unsafe { SHGetFolderPathW(null_mut(), csidl as i32, null_mut(), 0, out) }
    }
}

/// `SHGetKnownFolderPath(FOLDERID_RoamingAppData)` copied to `out`; returns the length, 0 on failure.
///
/// # Safety
/// `out` is valid for `cap` UTF-16 units.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn probe_known_folder_roaming(out: *mut u16, cap: u32) -> u32 {
    // SAFETY: forwarded from the caller.
    unsafe { Probe::known_folder(&FOLDERID_RoamingAppData, out, cap) }
}

/// `SHGetKnownFolderPath(FOLDERID_LocalAppData)` copied to `out`; returns the length, 0 on failure.
///
/// # Safety
/// `out` is valid for `cap` UTF-16 units.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn probe_known_folder_local(out: *mut u16, cap: u32) -> u32 {
    // SAFETY: forwarded from the caller.
    unsafe { Probe::known_folder(&FOLDERID_LocalAppData, out, cap) }
}

/// `SHGetFolderPathW(CSIDL_APPDATA)` into `out`; returns the HRESULT.
///
/// # Safety
/// `out` is valid for `MAX_PATH` UTF-16 units.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn probe_folder_path_appdata(out: *mut u16) -> i32 {
    // SAFETY: forwarded from the caller.
    unsafe { Probe::folder_path(CSIDL_APPDATA, out) }
}

/// `SHGetFolderPathW(CSIDL_PERSONAL)` into `out`; returns the HRESULT.
///
/// # Safety
/// `out` is valid for `MAX_PATH` UTF-16 units.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn probe_folder_path_personal(out: *mut u16) -> i32 {
    // SAFETY: forwarded from the caller.
    unsafe { Probe::folder_path(CSIDL_PERSONAL, out) }
}

/// `GetEnvironmentVariableW(name)` into `out`; returns the call's own result.
///
/// # Safety
/// `name` is NUL-terminated and `out` is valid for `cap` UTF-16 units.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn probe_env(name: *const u16, out: *mut u16, cap: u32) -> u32 {
    // SAFETY: forwarded from the caller.
    unsafe { GetEnvironmentVariableW(name, out, cap) }
}

/// `GetEnvironmentVariableW("APPDATA")` into `out`; returns the call's own result.
///
/// # Safety
/// `out` is valid for `cap` UTF-16 units.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn probe_env_appdata(out: *mut u16, cap: u32) -> u32 {
    let name: Vec<u16> = "APPDATA".encode_utf16().chain(std::iter::once(0)).collect();
    // SAFETY: `name` is NUL-terminated; the rest is forwarded from the caller.
    unsafe { GetEnvironmentVariableW(name.as_ptr(), out, cap) }
}

/// `GetTempPathW` into `out`; returns the call's own result.
///
/// # Safety
/// `out` is valid for `cap` UTF-16 units.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn probe_temp_path(out: *mut u16, cap: u32) -> u32 {
    // SAFETY: forwarded from the caller.
    unsafe { GetTempPathW(cap, out) }
}

/// The `MAX_PATH` the folder-path probes expect their buffers to hold.
#[unsafe(no_mangle)]
pub extern "C" fn probe_max_path() -> u32 {
    MAX_PATH
}

/// `GetEnvironmentVariableW("APPDATA")` with a NULL buffer and size 0; returns the required size.
#[unsafe(no_mangle)]
pub extern "C" fn probe_env_size() -> u32 {
    let name: Vec<u16> = "APPDATA".encode_utf16().chain(std::iter::once(0)).collect();
    // SAFETY: `name` is NUL-terminated; a NULL buffer with size 0 is the documented size query.
    unsafe { GetEnvironmentVariableW(name.as_ptr(), null_mut(), 0) }
}

/// `GetEnvironmentVariableA(name)` into `out`; returns the call's own result.
///
/// # Safety
/// `name` is NUL-terminated and `out` is valid for `cap` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn probe_env_a(name: *const u8, out: *mut u8, cap: u32) -> u32 {
    // SAFETY: forwarded from the caller.
    unsafe { GetEnvironmentVariableA(name, out, cap) }
}

/// `GetTempPathA` into `out`; returns the call's own result.
///
/// # Safety
/// `out` is valid for `cap` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn probe_temp_path_a(out: *mut u8, cap: u32) -> u32 {
    // SAFETY: forwarded from the caller.
    unsafe { GetTempPathA(cap, out) }
}

/// `getenv(name)` copied to `out`; returns the length, or `u32::MAX` when the variable is unset.
///
/// # Safety
/// `name` is NUL-terminated and `out` is valid for `cap` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn probe_getenv(name: *const c_char, out: *mut u8, cap: u32) -> u32 {
    // SAFETY: `name` is NUL-terminated.
    let value = unsafe { getenv(name) };
    if value.is_null() {
        return u32::MAX;
    }
    // SAFETY: `getenv` returns a NUL-terminated string; `out` holds `cap` bytes.
    unsafe { Probe::copy_narrow(value.cast(), out, cap) }
}

/// `getenv_s(name)` into `out`; returns the errno result.
///
/// # Safety
/// `name` is NUL-terminated and `out` is valid for `cap` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn probe_getenv_s(name: *const c_char, out: *mut u8, cap: u32) -> i32 {
    let mut size = 0usize;
    // SAFETY: `size` is a valid out pointer; the rest is forwarded from the caller.
    unsafe { getenv_s(&mut size, out.cast(), cap as usize, name) }
}

/// `_dupenv_s(name)` copied to `out`; returns the errno result.
///
/// # Safety
/// `name` is NUL-terminated and `out` is valid for `cap` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn probe_dupenv_s(name: *const c_char, out: *mut u8, cap: u32) -> i32 {
    let mut buffer: *mut c_char = null_mut();
    let mut count = 0usize;
    // SAFETY: both out pointers are valid.
    let status = unsafe { _dupenv_s(&mut buffer, &mut count, name) };
    if !buffer.is_null() {
        // SAFETY: the CRT returned a NUL-terminated string that this call frees once.
        unsafe {
            Probe::copy_narrow(buffer.cast(), out, cap);
            free(buffer.cast());
        }
    }
    status
}

/// `_wdupenv_s(name)` copied to `out`; returns the errno result.
///
/// # Safety
/// `name` is NUL-terminated and `out` is valid for `cap` UTF-16 units.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn probe_wdupenv_s(name: *const u16, out: *mut u16, cap: u32) -> i32 {
    let mut buffer: *mut u16 = null_mut();
    let mut count = 0usize;
    // SAFETY: both out pointers are valid.
    let status = unsafe { _wdupenv_s(&mut buffer, &mut count, name) };
    if !buffer.is_null() {
        // SAFETY: the CRT returned a NUL-terminated string that this call frees once.
        unsafe {
            Probe::copy_out(buffer, out, cap);
            free(buffer.cast());
        }
    }
    status
}
