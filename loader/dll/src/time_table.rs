use amml_abi::{AmmlHoldRequest, AmmlStatus, AmmlStr, AmmlTimeHold, AmmlTimeState, AmmlTimeTable};

use crate::{Boundary, LoaderState, TimeAdapter};

const TABLE_VERSION: u32 = 1;

pub(crate) static TIME: AmmlTimeTable = AmmlTimeTable {
    size: AmmlTimeTable::SIZE_V1,
    version: TABLE_VERSION,
    hold: TimeTable::hold,
    step_us: TimeTable::step_us,
    release: TimeTable::release,
    state: TimeTable::state,
};

/// The time table behind `AmmlServices::time`, and its `extern "C"` functions. Each function checks its pointers and
/// delegates to `TimeAdapter` inside the boundary, so a panic never crosses it. With no adapter installed every call
/// gives `NOT_AVAILABLE`.
pub struct TimeTable;

impl TimeTable {
    unsafe extern "C" fn hold(module: AmmlStr, request: *const AmmlHoldRequest, out: *mut AmmlTimeHold) -> AmmlStatus {
        Boundary::guard(|| {
            if request.is_null() || out.is_null() {
                return AmmlStatus::INVALID_ARGUMENT;
            }
            let owner = match Self::owner(module) {
                Ok(owner) => owner,
                Err(status) => return status,
            };
            // SAFETY: `request` is non-null and the caller promises it is readable for the call.
            let request = unsafe { request.read() };
            match TimeAdapter::get().ok_or(AmmlStatus::NOT_AVAILABLE).and_then(|a| a.hold(owner, request)) {
                Ok(hold) => {
                    // SAFETY: `out` is non-null and the caller promises it is writable.
                    unsafe { out.write(hold) };
                    AmmlStatus::OK
                }
                Err(status) => status,
            }
        })
    }

    unsafe extern "C" fn step_us(hold: AmmlTimeHold, micros: i64) -> AmmlStatus {
        Boundary::guard(|| {
            match TimeAdapter::get().ok_or(AmmlStatus::NOT_AVAILABLE).and_then(|a| a.step_us(hold, micros)) {
                Ok(()) => AmmlStatus::OK,
                Err(status) => status,
            }
        })
    }

    unsafe extern "C" fn release(hold: AmmlTimeHold) -> AmmlStatus {
        Boundary::guard(|| match TimeAdapter::get().ok_or(AmmlStatus::NOT_AVAILABLE).and_then(|a| a.release(hold)) {
            Ok(()) => AmmlStatus::OK,
            Err(status) => status,
        })
    }

    unsafe extern "C" fn state(out: *mut AmmlTimeState) -> AmmlStatus {
        Boundary::guard(|| {
            if out.is_null() {
                return AmmlStatus::INVALID_ARGUMENT;
            }
            let Some(adapter) = TimeAdapter::get().filter(|adapter| adapter.is_offered()) else {
                return AmmlStatus::NOT_AVAILABLE;
            };
            // SAFETY: `out` is non-null and the caller promises it is writable.
            unsafe { out.write(adapter.state()) };
            AmmlStatus::OK
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
