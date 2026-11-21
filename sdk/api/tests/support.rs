use std::ffi::c_void;

use amml_abi::{AmmlApi, AmmlEventHeader, AmmlLogLevel, AmmlStatus, AmmlStr};
use amml_api::Event;

/// Loader tables with v1 fields only; the struct has the v3 layout, `size` tells how much counts.
pub struct ShortTable;

impl ShortTable {
    unsafe extern "C" fn resolve(_: AmmlStr, _: *mut u64) -> AmmlStatus {
        AmmlStatus::FAILED
    }

    unsafe extern "C" fn hook(
        _: *const c_void,
        _: *const c_void,
        _: i32,
        _: *mut *const c_void,
        _: *mut u64,
    ) -> AmmlStatus {
        AmmlStatus::FAILED
    }

    unsafe extern "C" fn unhook(_: u64) -> AmmlStatus {
        AmmlStatus::FAILED
    }

    unsafe extern "C" fn log(_: AmmlStr, _: AmmlLogLevel, _: AmmlStr) {}

    unsafe extern "C" fn heap_new(_: usize) -> *mut u8 {
        std::ptr::null_mut()
    }

    unsafe extern "C" fn heap_delete(_: *mut u8, _: usize) {}

    unsafe extern "C" fn mod_dir(_: AmmlStr, _: u8, _: *mut AmmlStr) -> AmmlStatus {
        AmmlStatus::FAILED
    }

    pub fn table(size: u32, version: u32) -> AmmlApi {
        AmmlApi {
            size,
            version,
            module_base: 0,
            resolve: Self::resolve,
            hook: Self::hook,
            unhook: Self::unhook,
            log: Self::log,
            heap_new: Self::heap_new,
            heap_delete: Self::heap_delete,
            mod_dir: Self::mod_dir,
            services: std::ptr::null(),
        }
    }
}

/// A cancellable event for tests; no event of API v3 can be cancelled yet.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct CancellableProbe {
    pub header: AmmlEventHeader,
}

// SAFETY: `#[repr(C)]` and starts with an `AmmlEventHeader`; the fake delivers this struct for `ID`.
#[allow(unsafe_code)]
unsafe impl Event for CancellableProbe {
    const ID: &'static str = "test.cancellable";
    const VERSION: u32 = 1;
    const CANCELLABLE: bool = true;
}
