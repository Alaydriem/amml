use amml_abi::{AmmlCaptureFrame, AmmlOverlayTable};

use crate::CaptureFrame;

/// An open capture stream, from `Overlay::open_capture_stream`. Dropping it closes the stream; the
/// loader also closes it when the mod is disabled. Usable from any thread.
#[must_use = "dropping the stream closes it"]
pub struct CaptureStream {
    table: AmmlOverlayTable,
    id: u64,
}

impl CaptureStream {
    pub(crate) fn new(table: AmmlOverlayTable, id: u64) -> Self {
        Self { table, id }
    }

    /// The loader's id of the stream.
    pub fn id(&self) -> u64 {
        self.id
    }

    /// The oldest filled frame, until the returned frame is dropped; `None` when none waits or the
    /// stream is closed. One frame is taken at a time.
    pub fn take(&mut self) -> Option<CaptureFrame<'_>> {
        let mut frame = AmmlCaptureFrame {
            pixels: std::ptr::null(),
            len: 0,
            width: 0,
            height: 0,
            stride: 0,
            _reserved: 0,
            present_index: 0,
        };
        // SAFETY: the loader accepts any stream id, and `frame` is a live local.
        #[allow(unsafe_code)]
        let status = unsafe { (self.table.capture_take)(self.id, &mut frame) };
        if !status.is_ok() {
            return None;
        }
        // SAFETY: the loader keeps the frame's pixels until it gets the frame back or the stream is
        // closed; the frame borrows the stream mutably, so neither happens while it lives.
        #[allow(unsafe_code)]
        let taken = unsafe { CaptureFrame::new(self.table.capture_give_back, self.id, frame) };
        if taken.is_none() {
            // SAFETY: the loader accepts any stream id and frame index.
            #[allow(unsafe_code)]
            let _ = unsafe { (self.table.capture_give_back)(self.id, frame.present_index) };
        }
        taken
    }

    /// The slots the loader can fill now; 0 for a closed stream.
    pub fn free_slots(&self) -> u32 {
        // SAFETY: the loader accepts any stream id and gives 0 for an unknown one.
        #[allow(unsafe_code)]
        unsafe {
            (self.table.capture_free_slots)(self.id)
        }
    }
}

impl Drop for CaptureStream {
    fn drop(&mut self) {
        // SAFETY: the loader accepts any stream id and gives `STALE` for a closed one.
        #[allow(unsafe_code)]
        let _ = unsafe { (self.table.close_capture)(self.id) };
    }
}
