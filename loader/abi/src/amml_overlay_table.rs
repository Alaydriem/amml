use core::ffi::c_void;

use crate::{AmmlCaptureFrame, AmmlCaptureRequest, AmmlDrawSurface, AmmlShotLayers, AmmlStatus, AmmlStr};

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
/// Makes this frame's draw surface when it was not made yet and writes it to `out`. Render thread, inside a
/// `FramePresent` raise only; `REFUSED` outside one, `NOT_READY` when the frame can have no surface (no device yet,
/// no target could be made, a 0 x 0 chain). In a frame whose surface was made before the listeners (an older
/// listener, a capture due) it gives that surface. For a listener at `FramePresent::ON_DEMAND_VERSION`.
pub type OverlayPrepareSurfaceFn = unsafe extern "C" fn(out: *mut AmmlDrawSurface) -> AmmlStatus;

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
    pub prepare_surface: Option<OverlayPrepareSurfaceFn>,
}

impl AmmlOverlayTable {
    /// `size` of a v1 overlay table. A literal, so it keeps its meaning when a later version adds fields.
    pub const SIZE_V1: u32 = 72;
    /// `size` of a v2 overlay table: v1 plus `prepare_surface`.
    pub const SIZE_V2: u32 = 80;

    /// A copy of the table at `table`, read as far as its own `size` covers; a later field is `None`. A null table, or
    /// one shorter than v1, gives `None`.
    ///
    /// # Safety
    /// A non-null `table` points to a table readable for its own `size` bytes, at least 4.
    #[allow(unsafe_code)]
    pub unsafe fn read(table: *const AmmlOverlayTable) -> Option<AmmlOverlayTable> {
        if table.is_null() {
            return None;
        }
        // SAFETY: the caller guarantees the leading `size` field is readable.
        let size = unsafe { table.cast::<u32>().read_unaligned() };
        if size < Self::SIZE_V1 {
            return None;
        }
        let covered = (size as usize).min(core::mem::size_of::<AmmlOverlayTable>());
        let mut copy = core::mem::MaybeUninit::<AmmlOverlayTable>::zeroed();
        // SAFETY: the caller guarantees `covered` bytes are readable at `table`; `copy` has room for the whole struct
        // and the two do not overlap.
        unsafe { core::ptr::copy_nonoverlapping(table.cast::<u8>(), copy.as_mut_ptr().cast::<u8>(), covered) };
        // SAFETY: the copied bytes cover every v1 field, written from a valid table; every later field is an `Option`
        // of a function pointer, for which all zero bytes are `None`.
        Some(unsafe { copy.assume_init() })
    }
}

const _: () = assert!(core::mem::size_of::<AmmlOverlayTable>() == AmmlOverlayTable::SIZE_V2 as usize);
