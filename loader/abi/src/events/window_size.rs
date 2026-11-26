use crate::{AmmlEvent, AmmlEventHeader};

/// The client area of the game window changed size. A listener may write `width` and `height`; the game then
/// sees the written size.
///
/// Raised on thread `WINDOW`, or `GAME` when the window thread is the game thread.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct WindowSize {
    pub header: AmmlEventHeader,
    pub minimised: u8,
    pub _reserved: [u8; 3],
    pub width: u32,
    pub height: u32,
}

impl AmmlEvent for WindowSize {
    const ID: &'static str = "window.size";
}

const _: () = assert!(core::mem::size_of::<WindowSize>() == 28);
