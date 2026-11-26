use crate::{AmmlEvent, AmmlEventHeader};

/// The image of screenshot `id` was copied from the back buffer; `ok` is 1 on success.
///
/// Raised on thread `GAME`, to the mod that asked for the screenshot.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ScreenshotTaken {
    pub header: AmmlEventHeader,
    pub id: u64,
    pub ok: u8,
    pub _reserved: [u8; 7],
}

impl AmmlEvent for ScreenshotTaken {
    const ID: &'static str = "overlay.screenshot_taken";
}

const _: () = assert!(core::mem::size_of::<ScreenshotTaken>() == 32);
