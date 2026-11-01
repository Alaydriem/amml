use std::ffi::c_void;

use amml_abi::{AMML_API_VERSION, AmmlApi, AmmlLogLevel, AmmlStatus, AmmlStr, KnownLogLevel};

use crate::{Boundary, LoaderLog, LoaderState};

/// The `extern "C"` functions of the `AmmlApi` table. Each is a thin delegator into
/// `LoaderState` inside `Boundary::guard`, so a panic becomes `AmmlStatus::PANICKED`.
pub struct ApiTable;

impl ApiTable {
    pub fn build(module_base: u64) -> AmmlApi {
        AmmlApi {
            size: AmmlApi::SIZE_V1,
            version: AMML_API_VERSION,
            module_base,
            resolve: Self::resolve,
            hook: Self::hook,
            unhook: Self::unhook,
            log: Self::log,
            heap_new: Self::heap_new,
            heap_delete: Self::heap_delete,
            mod_dir: Self::mod_dir,
        }
    }

    unsafe extern "C" fn resolve(id: AmmlStr, out: *mut u64) -> AmmlStatus {
        Boundary::guard(|| {
            let Some(state) = LoaderState::get() else {
                return AmmlStatus::FAILED;
            };
            if out.is_null() {
                return AmmlStatus::INVALID_ARGUMENT;
            }
            // SAFETY: the caller promises `id` is valid for the call.
            let id = match unsafe { id.as_str() } {
                Ok(id) => id,
                Err(status) => return status,
            };
            match state.resolve(id) {
                Ok(value) => {
                    // SAFETY: `out` is non-null and the caller promises it is writable.
                    unsafe { out.write(value) };
                    AmmlStatus::OK
                }
                Err(status) => status,
            }
        })
    }

    unsafe extern "C" fn hook(
        target: *const c_void,
        detour: *const c_void,
        priority: i32,
        original: *mut *const c_void,
        handle: *mut u64,
    ) -> AmmlStatus {
        Boundary::guard(|| {
            let Some(state) = LoaderState::get() else {
                return AmmlStatus::FAILED;
            };
            if handle.is_null() || target.is_null() || detour.is_null() || original.is_null() {
                return AmmlStatus::INVALID_ARGUMENT;
            }
            // SAFETY: the caller promises the target, detour and original slot as the contract of
            // `AmmlApi::hook` requires; null pointers were refused above.
            match unsafe { state.install_hook(target, detour, priority, original) } {
                Ok(value) => {
                    // SAFETY: `handle` is non-null and the caller promises it is writable.
                    unsafe { handle.write(value) };
                    AmmlStatus::OK
                }
                Err(status) => status,
            }
        })
    }

    unsafe extern "C" fn unhook(handle: u64) -> AmmlStatus {
        Boundary::guard(|| match LoaderState::get() {
            Some(state) => match state.remove_hook(handle) {
                Ok(()) => AmmlStatus::OK,
                Err(status) => status,
            },
            None => AmmlStatus::FAILED,
        })
    }

    unsafe extern "C" fn log(module: AmmlStr, level: AmmlLogLevel, message: AmmlStr) {
        Boundary::run((), || {
            // SAFETY: the caller promises both views are valid for the call.
            let (module, message) = unsafe { (module.as_str(), message.as_str()) };
            let level = level.known().unwrap_or(KnownLogLevel::Info);
            match (module, message) {
                (Ok(module), Ok(message)) => LoaderLog::module_line(module, level, message),
                _ => curia::warn!("a mod sent a log line that is not valid text"),
            }
        });
    }

    unsafe extern "C" fn heap_new(size: usize) -> *mut u8 {
        Boundary::run(std::ptr::null_mut(), || match LoaderState::get() {
            Some(state) => state.heap().alloc(size),
            None => std::ptr::null_mut(),
        })
    }

    unsafe extern "C" fn heap_delete(ptr: *mut u8, size: usize) {
        Boundary::run((), || {
            if let Some(state) = LoaderState::get() {
                // SAFETY: the caller promises `ptr` came from `heap_new` with this `size`.
                unsafe { state.heap().free(ptr, size) };
            }
        });
    }

    unsafe extern "C" fn mod_dir(module: AmmlStr, kind: u8, out: *mut AmmlStr) -> AmmlStatus {
        Boundary::guard(|| {
            let Some(state) = LoaderState::get() else {
                return AmmlStatus::FAILED;
            };
            if out.is_null() {
                return AmmlStatus::INVALID_ARGUMENT;
            }
            // SAFETY: the caller promises `module` is valid for the call.
            let module = match unsafe { module.as_str() } {
                Ok(module) => module,
                Err(status) => return status,
            };
            match state.mod_dir(module, kind) {
                Ok(view) => {
                    // SAFETY: `out` is non-null and the caller promises it is writable.
                    unsafe { out.write(view) };
                    AmmlStatus::OK
                }
                Err(status) => status,
            }
        })
    }
}
