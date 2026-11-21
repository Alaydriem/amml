use std::ffi::c_void;
use std::path::Path;

use amml_abi::{AmmlCaptureFrame, AmmlCaptureRequest, AmmlOverlayTable, AmmlShotLayers, AmmlStatus, AmmlStr};

use crate::{BackBufferCopy, Boundary, CaptureService, LoaderState, OverlayService};

const TABLE_VERSION: u32 = 1;

pub(crate) static OVERLAY: AmmlOverlayTable = AmmlOverlayTable {
    size: AmmlOverlayTable::SIZE_V1,
    version: TABLE_VERSION,
    screenshot: OverlayTable::screenshot,
    open_capture: OverlayTable::open_capture,
    capture_take: OverlayTable::capture_take,
    capture_give_back: OverlayTable::capture_give_back,
    capture_free_slots: OverlayTable::capture_free_slots,
    close_capture: OverlayTable::close_capture,
    present_count: OverlayTable::present_count,
    copy_back_buffer: OverlayTable::copy_back_buffer,
};

/// The overlay table behind `AmmlServices::overlay`, and its `extern "C"` functions. Each function checks its
/// pointers and delegates inside the boundary, so a panic never crosses it. While an area is not offered (no overlay
/// hooks, or the LeviLamina host), its functions give `NOT_AVAILABLE`: `overlay.surface` for `copy_back_buffer` (and
/// `present_count` gives 0), `overlay.screenshot` for `screenshot`, `overlay.capture` for the stream functions
/// (`capture_free_slots` gives 0).
pub struct OverlayTable;

impl OverlayTable {
    unsafe extern "C" fn screenshot(
        module: AmmlStr,
        path: AmmlStr,
        layers: AmmlShotLayers,
        out_id: *mut u64,
    ) -> AmmlStatus {
        Boundary::guard(|| {
            let state = match Self::area_state(CaptureService::SCREENSHOT_AREA) {
                Ok(state) => state,
                Err(status) => return status,
            };
            // SAFETY: the caller promises `module` and `path` are valid for the call.
            let (owner, path) = match (Self::owner(state, module), unsafe { path.as_str() }) {
                (Ok(owner), Ok(path)) => (owner, path),
                (Err(status), _) | (_, Err(status)) => return status,
            };
            if out_id.is_null() {
                return AmmlStatus::INVALID_ARGUMENT;
            }
            match state.capture().screenshot(owner, Path::new(path), layers) {
                Ok(id) => {
                    // SAFETY: `out_id` is non-null and the caller promises it is writable.
                    unsafe { out_id.write(id) };
                    AmmlStatus::OK
                }
                Err(status) => status,
            }
        })
    }

    unsafe extern "C" fn open_capture(
        module: AmmlStr,
        request: *const AmmlCaptureRequest,
        out_stream: *mut u64,
    ) -> AmmlStatus {
        Boundary::guard(|| {
            let state = match Self::area_state(CaptureService::CAPTURE_AREA) {
                Ok(state) => state,
                Err(status) => return status,
            };
            let owner = match Self::owner(state, module) {
                Ok(owner) => owner,
                Err(status) => return status,
            };
            if request.is_null() || out_stream.is_null() {
                return AmmlStatus::INVALID_ARGUMENT;
            }
            // SAFETY: `request` is non-null and the caller promises it points to a readable v1 request.
            let request = unsafe { request.read() };
            match state.capture().open_capture(owner, &request) {
                Ok(stream) => {
                    // SAFETY: `out_stream` is non-null and the caller promises it is writable.
                    unsafe { out_stream.write(stream) };
                    AmmlStatus::OK
                }
                Err(status) => status,
            }
        })
    }

    unsafe extern "C" fn capture_take(stream: u64, out: *mut AmmlCaptureFrame) -> AmmlStatus {
        Boundary::guard(|| {
            let state = match Self::area_state(CaptureService::CAPTURE_AREA) {
                Ok(state) => state,
                Err(status) => return status,
            };
            if out.is_null() {
                return AmmlStatus::INVALID_ARGUMENT;
            }
            match state.capture().take(stream) {
                Ok(frame) => {
                    // SAFETY: `out` is non-null and the caller promises it is writable.
                    unsafe { out.write(frame) };
                    AmmlStatus::OK
                }
                Err(status) => status,
            }
        })
    }

    unsafe extern "C" fn capture_give_back(stream: u64, present_index: u64) -> AmmlStatus {
        Boundary::guard(|| match Self::area_state(CaptureService::CAPTURE_AREA) {
            Ok(state) => Self::status(state.capture().give_back(stream, present_index)),
            Err(status) => status,
        })
    }

    unsafe extern "C" fn capture_free_slots(stream: u64) -> u32 {
        Boundary::run(0, || {
            Self::area_state(CaptureService::CAPTURE_AREA).map_or(0, |state| state.capture().free_slots(stream))
        })
    }

    unsafe extern "C" fn close_capture(stream: u64) -> AmmlStatus {
        Boundary::guard(|| match Self::area_state(CaptureService::CAPTURE_AREA) {
            Ok(state) => Self::status(state.capture().close(stream)),
            Err(status) => status,
        })
    }

    unsafe extern "C" fn present_count() -> u64 {
        Boundary::run(0, || if Self::available() { OverlayService::present_count() } else { 0 })
    }

    unsafe extern "C" fn copy_back_buffer(dest: *mut c_void) -> AmmlStatus {
        Boundary::guard(|| {
            if !Self::available() {
                return AmmlStatus::NOT_AVAILABLE;
            }
            // SAFETY: the caller promises a non-null `dest` is a live COM object for the call.
            unsafe { BackBufferCopy::copy(dest) }
        })
    }

    // The loader state when `area` is offered; `NOT_AVAILABLE` otherwise.
    fn area_state(area: &str) -> Result<&'static LoaderState, AmmlStatus> {
        LoaderState::get().filter(|state| state.areas().is_available(area, 1)).ok_or(AmmlStatus::NOT_AVAILABLE)
    }

    // The index of the live mod `module`.
    fn owner(state: &LoaderState, module: AmmlStr) -> Result<usize, AmmlStatus> {
        // SAFETY: the caller of the table function promises `module` is valid for the call.
        let module = unsafe { module.as_str() }?;
        state.live_mod(module)
    }

    fn status(result: Result<(), AmmlStatus>) -> AmmlStatus {
        result.err().unwrap_or(AmmlStatus::OK)
    }

    // The overlay hooks are in: the install offered the surface area.
    fn available() -> bool {
        LoaderState::get().is_some_and(|state| state.areas().is_available(OverlayService::SURFACE_AREA, 1))
    }
}
