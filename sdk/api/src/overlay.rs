use std::path::Path;

use amml_abi::{AmmlCaptureRequest, AmmlOverlayTable, AmmlShotLayers, AmmlStatus, AmmlStr};

use crate::{ApiError, CaptureRequest, CaptureStream, ShotId, ShotLayers};

/// Screenshots, capture streams and the present count of the loader's overlay. Usable from any thread.
///
/// Without an overlay table (a loader before v3) every call gives `NotAvailable` or 0. The loader
/// serves the overlay areas only on the standalone host, once its present hooks are installed.
#[derive(Clone, Copy)]
pub struct Overlay {
    table: Option<AmmlOverlayTable>,
    module: &'static str,
}

impl Overlay {
    pub(crate) fn new(table: Option<AmmlOverlayTable>, module: &'static str) -> Self {
        Self { table, module }
    }

    /// Asks for a PNG of the next frame that has a surface, written to `path`. The loader raises
    /// `ScreenshotTaken` (once the image is read back) and then `ScreenshotWritten` to this mod, each
    /// with the returned id; both come, with `ok` 0 on any failure, also when no frame with a surface
    /// comes within 120 presents or 5 s (minimised, no device) and when the loader shuts down before
    /// the copy. At shutdown a shot already copied is lost when its read-back or its file is not done
    /// in time. `path` must be absolute (else `Loader(INVALID_ARGUMENT)`, also for a path that is not
    /// UTF-8); its folder is not made. The file is written as `<path>.tmp` and renamed. `NotReady`
    /// before the game's swap chain is bound and while the loader shuts down; `NotAvailable` without
    /// the area `overlay.screenshot`.
    pub fn screenshot(&self, path: &Path, layers: ShotLayers) -> Result<ShotId, ApiError> {
        let table = self.table.ok_or(ApiError::NotAvailable)?;
        let path = path.to_str().ok_or(ApiError::Loader(AmmlStatus::INVALID_ARGUMENT))?;
        let mut id = 0u64;
        // SAFETY: the module id and the path live for the call, and `id` is a live local.
        #[allow(unsafe_code)]
        let status = unsafe {
            (table.screenshot)(
                AmmlStr::from_str(self.module),
                AmmlStr::from_str(path),
                AmmlShotLayers::from(layers),
                &mut id,
            )
        };
        ApiError::check(status).map(|()| ShotId(id))
    }

    /// Opens a capture stream: the loader copies every `every_nth_present`th present into a free slot of
    /// the stream's ring and skips a present when no slot is free (it never waits). A `ring` outside 1
    /// to 8 gives `Loader(INVALID_ARGUMENT)`; `NotReady` before the game's swap chain is bound and
    /// while the loader shuts down; `NotAvailable` without the area `overlay.capture`. Dropping the stream closes it.
    pub fn open_capture_stream(&self, request: CaptureRequest) -> Result<CaptureStream, ApiError> {
        let table = self.table.ok_or(ApiError::NotAvailable)?;
        let limit = request.max_size.unwrap_or_default();
        let abi = AmmlCaptureRequest {
            size: AmmlCaptureRequest::SIZE_V1,
            layers: AmmlShotLayers::from(request.layers),
            ring: request.ring,
            _reserved: 0,
            every_nth_present: request.every_nth_present,
            max_width: limit.width,
            max_height: limit.height,
        };
        let mut stream = 0u64;
        // SAFETY: the module id and `abi` live for the call, and `stream` is a live local.
        #[allow(unsafe_code)]
        let status = unsafe { (table.open_capture)(AmmlStr::from_str(self.module), &abi, &mut stream) };
        ApiError::check(status).map(|()| CaptureStream::new(table, stream))
    }

    /// The presents of the game's swap chain since the loader bound it; 0 without the overlay. It is
    /// also the `frame_id` of `FramePresent` and the `present_index` of capture frames.
    pub fn present_count(&self) -> u64 {
        let Some(table) = &self.table else {
            return 0;
        };
        // SAFETY: the loader's function takes no arguments.
        #[allow(unsafe_code)]
        unsafe {
            (table.present_count)()
        }
    }
}
