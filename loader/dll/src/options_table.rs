use std::ffi::c_void;

use amml_abi::{AmmlOptionKey, AmmlOptionsTable, AmmlPersistence, AmmlStatus, AmmlStr};

use crate::{Boundary, LoaderState, OptionsAdapter};

const TABLE_VERSION: u32 = 1;

pub(crate) static OPTIONS: AmmlOptionsTable = AmmlOptionsTable {
    size: AmmlOptionsTable::SIZE_V1,
    version: TABLE_VERSION,
    view_distance_chunks: OptionsTable::view_distance_chunks,
    get_int: OptionsTable::get_int,
    set_int: OptionsTable::set_int,
    release_transient: OptionsTable::release_transient,
};

/// The options table behind `AmmlServices::options`, and its `extern "C"` functions. Each function checks its pointers
/// and delegates to `OptionsAdapter` inside the boundary, so a panic never crosses it. With no adapter installed every
/// call gives `NOT_AVAILABLE`. Every call is for the game thread; the adapter refuses any other.
pub struct OptionsTable;

impl OptionsTable {
    unsafe extern "C" fn view_distance_chunks(client: *const c_void, out: *mut u32) -> AmmlStatus {
        Boundary::guard(|| {
            if out.is_null() {
                return AmmlStatus::INVALID_ARGUMENT;
            }
            match OptionsAdapter::get().ok_or(AmmlStatus::NOT_AVAILABLE).and_then(|a| a.view_distance_chunks(client)) {
                Ok(chunks) => {
                    // SAFETY: `out` is non-null and the caller promises it is writable.
                    unsafe { out.write(chunks) };
                    AmmlStatus::OK
                }
                Err(status) => status,
            }
        })
    }

    unsafe extern "C" fn get_int(key: AmmlOptionKey, out: *mut i32) -> AmmlStatus {
        Boundary::guard(|| {
            if out.is_null() {
                return AmmlStatus::INVALID_ARGUMENT;
            }
            match OptionsAdapter::get().ok_or(AmmlStatus::NOT_AVAILABLE).and_then(|a| a.get_int(key)) {
                Ok(value) => {
                    // SAFETY: `out` is non-null and the caller promises it is writable.
                    unsafe { out.write(value) };
                    AmmlStatus::OK
                }
                Err(status) => status,
            }
        })
    }

    unsafe extern "C" fn set_int(
        module: AmmlStr,
        key: AmmlOptionKey,
        value: i32,
        persistence: AmmlPersistence,
    ) -> AmmlStatus {
        Boundary::guard(|| {
            let owner = match Self::owner(module) {
                Ok(owner) => owner,
                Err(status) => return status,
            };
            let set = OptionsAdapter::get()
                .ok_or(AmmlStatus::NOT_AVAILABLE)
                .and_then(|adapter| adapter.set_int(owner, key, value, persistence));
            match set {
                Ok(()) => AmmlStatus::OK,
                Err(status) => status,
            }
        })
    }

    unsafe extern "C" fn release_transient(module: AmmlStr, key: AmmlOptionKey) -> AmmlStatus {
        Boundary::guard(|| {
            let owner = match Self::owner(module) {
                Ok(owner) => owner,
                Err(status) => return status,
            };
            let released = OptionsAdapter::get()
                .ok_or(AmmlStatus::NOT_AVAILABLE)
                .and_then(|adapter| adapter.release_transient(owner, key));
            match released {
                Ok(()) => AmmlStatus::OK,
                Err(status) => status,
            }
        })
    }

    // The index of the live mod `module`.
    fn owner(module: AmmlStr) -> Result<usize, AmmlStatus> {
        let state = LoaderState::get().ok_or(AmmlStatus::FAILED)?;
        // SAFETY: the caller promises `module` is valid for the call.
        let module = unsafe { module.as_str() }?;
        state.live_mod(module)
    }
}
