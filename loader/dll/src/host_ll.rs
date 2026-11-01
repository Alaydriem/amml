//! The exports LeviLamina calls on a native mod. The argument is LeviLamina's `Mod&`, which the
//! loader does not use. In this host `DllMain` returns at once (see `EntryPoint`), so nothing runs
//! under the OS loader lock.

use std::ffi::c_void;

use crate::{Boundary, HostStartup};

/// LeviLamina loads a native mod only when it exports this name; it marks a mod whose C++ new/delete
/// use LeviLamina's allocator. amml.dll has no C++ operators, never frees memory LeviLamina allocated
/// and never gives LeviLamina memory to free; what it hands to the game comes from the game's own
/// operator new. On the client LeviLamina does not replace the game allocator.
#[allow(non_upper_case_globals)]
#[unsafe(no_mangle)]
pub static ll_memory_operator_overrided: i32 = 0;

#[unsafe(no_mangle)]
pub extern "C" fn ll_mod_load(_mod: *mut c_void) -> bool {
    Boundary::run(false, HostStartup::load)
}

#[unsafe(no_mangle)]
pub extern "C" fn ll_mod_enable(_mod: *mut c_void) -> bool {
    Boundary::run(false, HostStartup::enable)
}

#[unsafe(no_mangle)]
pub extern "C" fn ll_mod_disable(_mod: *mut c_void) -> bool {
    Boundary::run(false, HostStartup::disable)
}

#[unsafe(no_mangle)]
pub extern "C" fn ll_mod_unload(_mod: *mut c_void) -> bool {
    Boundary::run(false, HostStartup::unload)
}
