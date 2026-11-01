//! The exports of the standalone host. `amml-patch` adds `amml.dll!amml_anchor` to the game exe's
//! import table, so Windows loads `amml.dll` before the exe runs. `DllMain` only installs the
//! entry-point detour (`EntryPoint`); in the LeviLamina host it does nothing.

use std::ffi::c_void;
use std::panic::catch_unwind;

use windows_sys::Win32::System::SystemServices::DLL_PROCESS_ATTACH;

use crate::EntryPoint;

/// Exists only to be imported by the patched exe.
#[unsafe(no_mangle)]
pub extern "system" fn amml_anchor() {}

/// Installs the entry-point detour only for a static load (`reserved` is non-null on
/// `DLL_PROCESS_ATTACH`), which is how the patched exe's import table loads `amml.dll`: the main
/// thread has not reached the entry point yet. A dynamic load (`LoadLibrary`, `reserved` null) comes
/// after the entry point ran, so a detour there would never fire and would only patch live code.
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub extern "system" fn DllMain(_module: *mut c_void, reason: u32, reserved: *mut c_void) -> i32 {
    if reason == DLL_PROCESS_ATTACH && !reserved.is_null() {
        let _ = catch_unwind(EntryPoint::install_if_standalone);
    }
    1
}
