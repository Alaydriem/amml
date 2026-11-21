use crate::{AmmlEvent, AmmlEventHeader};

/// Screenshot `id` was written to its file; `ok` is 0 when the file could not be written.
///
/// Raised on thread `GAME`, to the mod that asked for the screenshot.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ScreenshotWritten {
    pub header: AmmlEventHeader,
    pub id: u64,
    pub ok: u8,
    pub _reserved: [u8; 7],
}

impl AmmlEvent for ScreenshotWritten {
    const ID: &'static str = "overlay.screenshot_written";
}

const _: () = assert!(core::mem::size_of::<ScreenshotWritten>() == 32);
