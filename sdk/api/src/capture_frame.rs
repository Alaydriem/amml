use std::marker::PhantomData;

use amml_abi::{AmmlCaptureFrame, OverlayCaptureGiveBackFn};

use crate::{CaptureStream, Size};

/// One taken frame of a `CaptureStream`: BGRA8 pixels with alpha 255, `stride` bytes per row, scaled
/// to fit the stream's `max_size`. Dropping it gives the slot back to the loader.
pub struct CaptureFrame<'s> {
    give_back: OverlayCaptureGiveBackFn,
    stream: u64,
    frame: AmmlCaptureFrame,
    _stream: PhantomData<&'s mut CaptureStream>,
}

impl CaptureFrame<'_> {
    /// The frame `frame` of `stream`; `None` for null pixels, a `stride` below `width * 4` or fewer
    /// than `stride * height` bytes.
    ///
    /// # Safety
    /// A non-null `frame.pixels` is readable for `frame.len` bytes until the frame is given back or
    /// the stream is closed, and neither happens while the returned frame lives.
    #[allow(unsafe_code)]
    pub(crate) unsafe fn new(
        give_back: OverlayCaptureGiveBackFn,
        stream: u64,
        frame: AmmlCaptureFrame,
    ) -> Option<Self> {
        let row = u64::from(frame.width) * 4;
        let needed = u64::from(frame.stride) * u64::from(frame.height);
        let valid = !frame.pixels.is_null() && u64::from(frame.stride) >= row && frame.len as u64 >= needed;
        valid.then_some(Self { give_back, stream, frame, _stream: PhantomData })
    }

    /// The pixels: rows top to bottom, `stride()` bytes each, BGRA8 with alpha 255.
    pub fn pixels(&self) -> &[u8] {
        // SAFETY: `new` refused null pixels, and its contract keeps `len` bytes readable while `self`
        // lives.
        #[allow(unsafe_code)]
        unsafe {
            std::slice::from_raw_parts(self.frame.pixels, self.frame.len)
        }
    }

    pub fn size(&self) -> Size {
        Size { width: self.frame.width, height: self.frame.height }
    }

    /// The bytes per row.
    pub fn stride(&self) -> u32 {
        self.frame.stride
    }

    /// The present the frame was copied at (`Overlay::present_count` then). A gap larger than the
    /// stream's `every_nth_present` between two frames counts presents the stream skipped.
    pub fn present_index(&self) -> u64 {
        self.frame.present_index
    }
}

impl Drop for CaptureFrame<'_> {
    fn drop(&mut self) {
        // SAFETY: the loader accepts any stream id and frame index.
        #[allow(unsafe_code)]
        let _ = unsafe { (self.give_back)(self.stream, self.frame.present_index) };
    }
}
