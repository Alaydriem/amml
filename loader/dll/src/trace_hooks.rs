use std::ffi::c_char;
use std::panic::{AssertUnwindSafe, catch_unwind};

use windows_sys::core::{PCWSTR, PWSTR};

use crate::folder_trace::FolderTrace;
use crate::{FolderCall, FolderRedirect};

type TempPathFn = unsafe extern "system" fn(u32, PWSTR) -> u32;
type TempPathAFn = unsafe extern "system" fn(u32, *mut u8) -> u32;
type EnvironmentAFn = unsafe extern "system" fn(*const u8, *mut u8, u32) -> u32;
type GetenvFn = unsafe extern "C" fn(*const c_char) -> *mut c_char;
type GetenvSFn = unsafe extern "C" fn(*mut usize, *mut c_char, usize, *const c_char) -> i32;
type DupenvSFn = unsafe extern "C" fn(*mut *mut c_char, *mut usize, *const c_char) -> i32;
type WdupenvSFn = unsafe extern "C" fn(*mut *mut u16, *mut usize, PCWSTR) -> i32;

/// Pass-through replacements that only log the call, for the routes to the data folder that
/// `FolderRedirect` does not redirect. Compiled only with the `folder-trace` feature.
pub struct TraceHooks;

impl TraceHooks {
    /// The kind and replacement address for a trace-only function.
    pub fn replacement(function: &str) -> Option<(FolderCall, usize)> {
        let found = match function {
            "GetTempPathW" => (FolderCall::TempPath, Self::temp_path as *const () as usize),
            "GetTempPathA" => (FolderCall::TempPathA, Self::temp_path_a as *const () as usize),
            "GetEnvironmentVariableA" => (FolderCall::EnvironmentA, Self::environment_a as *const () as usize),
            "getenv" => (FolderCall::Getenv, Self::getenv as *const () as usize),
            "getenv_s" => (FolderCall::GetenvS, Self::getenv_s as *const () as usize),
            "_dupenv_s" => (FolderCall::DupenvS, Self::dupenv_s as *const () as usize),
            "_wdupenv_s" => (FolderCall::WdupenvS, Self::wdupenv_s as *const () as usize),
            _ => return None,
        };
        Some(found)
    }

    // A trace must never stop the game.
    fn guarded(trace: impl FnOnce()) {
        let _ = catch_unwind(AssertUnwindSafe(trace));
    }

    extern "system" fn temp_path(size: u32, buffer: PWSTR) -> u32 {
        Self::guarded(|| FolderTrace::plain(FolderCall::TempPath));
        match FolderRedirect::original::<TempPathFn>(FolderCall::TempPath) {
            // SAFETY: the original takes the arguments the caller passed to this replacement.
            Some(original) => unsafe { original(size, buffer) },
            None => 0,
        }
    }

    extern "system" fn temp_path_a(size: u32, buffer: *mut u8) -> u32 {
        Self::guarded(|| FolderTrace::plain(FolderCall::TempPathA));
        match FolderRedirect::original::<TempPathAFn>(FolderCall::TempPathA) {
            // SAFETY: the original takes the arguments the caller passed to this replacement.
            Some(original) => unsafe { original(size, buffer) },
            None => 0,
        }
    }

    extern "system" fn environment_a(name: *const u8, buffer: *mut u8, size: u32) -> u32 {
        Self::guarded(|| FolderTrace::narrow(FolderCall::EnvironmentA, name));
        match FolderRedirect::original::<EnvironmentAFn>(FolderCall::EnvironmentA) {
            // SAFETY: the original takes the arguments the caller passed to this replacement.
            Some(original) => unsafe { original(name, buffer, size) },
            None => 0,
        }
    }

    extern "C" fn getenv(name: *const c_char) -> *mut c_char {
        Self::guarded(|| FolderTrace::narrow(FolderCall::Getenv, name.cast()));
        match FolderRedirect::original::<GetenvFn>(FolderCall::Getenv) {
            // SAFETY: the original takes the arguments the caller passed to this replacement.
            Some(original) => unsafe { original(name) },
            None => std::ptr::null_mut(),
        }
    }

    extern "C" fn getenv_s(size: *mut usize, buffer: *mut c_char, capacity: usize, name: *const c_char) -> i32 {
        Self::guarded(|| FolderTrace::narrow(FolderCall::GetenvS, name.cast()));
        match FolderRedirect::original::<GetenvSFn>(FolderCall::GetenvS) {
            // SAFETY: the original takes the arguments the caller passed to this replacement.
            Some(original) => unsafe { original(size, buffer, capacity, name) },
            None => Self::EINVAL,
        }
    }

    extern "C" fn dupenv_s(buffer: *mut *mut c_char, count: *mut usize, name: *const c_char) -> i32 {
        Self::guarded(|| FolderTrace::narrow(FolderCall::DupenvS, name.cast()));
        match FolderRedirect::original::<DupenvSFn>(FolderCall::DupenvS) {
            // SAFETY: the original takes the arguments the caller passed to this replacement.
            Some(original) => unsafe { original(buffer, count, name) },
            None => Self::EINVAL,
        }
    }

    extern "C" fn wdupenv_s(buffer: *mut *mut u16, count: *mut usize, name: PCWSTR) -> i32 {
        Self::guarded(|| FolderTrace::wide(FolderCall::WdupenvS, name));
        match FolderRedirect::original::<WdupenvSFn>(FolderCall::WdupenvS) {
            // SAFETY: the original takes the arguments the caller passed to this replacement.
            Some(original) => unsafe { original(buffer, count, name) },
            None => Self::EINVAL,
        }
    }

    const EINVAL: i32 = 22;
}
