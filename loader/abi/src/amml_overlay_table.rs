use core::ffi::c_void;

use crate::{AmmlCaptureFrame, AmmlCaptureRequest, AmmlShotLayers, AmmlStatus, AmmlStr};

/// Asks for a screenshot to `path` for `module` and gives its id in `out_id`; `ScreenshotTaken` and
/// `ScreenshotWritten` carry the id.
pub type OverlayScreenshotFn =
    unsafe extern "C" fn(module: AmmlStr, path: AmmlStr, layers: AmmlShotLayers, out_id: *mut u64) -> AmmlStatus;
/// Opens a capture stream for `module` and gives its id in `out_stream`.
pub type OverlayOpenCaptureFn =
    unsafe extern "C" fn(module: AmmlStr, request: *const AmmlCaptureRequest, out_stream: *mut u64) -> AmmlStatus;
/// Takes the oldest waiting frame of `stream` into `out`; `NOT_READY` when no frame waits.
pub type OverlayCaptureTakeFn = unsafe extern "C" fn(stream: u64, out: *mut AmmlCaptureFrame) -> AmmlStatus;
/// Gives the frame `present_index` of `stream` back to the loader.
pub type OverlayCaptureGiveBackFn = unsafe extern "C" fn(stream: u64, present_index: u64) -> AmmlStatus;
/// The slots of `stream` the loader can fill now; 0 for an unknown stream.
pub type OverlayCaptureFreeSlotsFn = unsafe extern "C" fn(stream: u64) -> u32;
/// Closes `stream` and frees its frames, also a taken one.
pub type OverlayCloseCaptureFn = unsafe extern "C" fn(stream: u64) -> AmmlStatus;
/// The presents of the bound swap chain so far.
pub type OverlayPresentCountFn = unsafe extern "C" fn() -> u64;
/// Copies the back buffer into the texture `dest`. Render thread, inside `FramePresent` only; `REFUSED` for a
/// texture of another size or format.
pub type OverlayCopyBackBufferFn = unsafe extern "C" fn(dest: *mut c_void) -> AmmlStatus;

/// The overlay functions of `AmmlServices`.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct AmmlOverlayTable {
    pub size: u32,
    pub version: u32,
    pub screenshot: OverlayScreenshotFn,
    pub open_capture: OverlayOpenCaptureFn,
    pub capture_take: OverlayCaptureTakeFn,
    pub capture_give_back: OverlayCaptureGiveBackFn,
    pub capture_free_slots: OverlayCaptureFreeSlotsFn,
    pub close_capture: OverlayCloseCaptureFn,
    pub present_count: OverlayPresentCountFn,
    pub copy_back_buffer: OverlayCopyBackBufferFn,
}

impl AmmlOverlayTable {
    /// `size` of a v1 overlay table. A literal, so it keeps its meaning when a later version adds fields.
    pub const SIZE_V1: u32 = 72;
}

const _: () = assert!(core::mem::size_of::<AmmlOverlayTable>() == AmmlOverlayTable::SIZE_V1 as usize);
