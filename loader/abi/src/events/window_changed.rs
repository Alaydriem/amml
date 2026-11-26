use crate::{AmmlEvent, AmmlEventHeader, AmmlWindowChange};

/// The DPI, the display or the position of the game window changed.
///
/// Raised on thread `WINDOW`, or `GAME` when the window thread is the game thread.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct WindowChanged {
    pub header: AmmlEventHeader,
    pub kind: AmmlWindowChange,
    pub _reserved: [u8; 3],
}

impl AmmlEvent for WindowChanged {
    const ID: &'static str = "window.changed";
}

const _: () = assert!(core::mem::size_of::<WindowChanged>() == 20);
