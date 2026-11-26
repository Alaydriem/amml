use crate::{AmmlCursorShape, AmmlEvent, AmmlEventHeader};

/// The game window sets its cursor. A listener that writes a `shape` other than `NONE` sets that shape, and the
/// game does not set its own.
///
/// Raised on thread `WINDOW`, or `GAME` when the window thread is the game thread.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct WindowCursor {
    pub header: AmmlEventHeader,
    pub shape: AmmlCursorShape,
    pub _reserved: [u8; 3],
}

impl AmmlEvent for WindowCursor {
    const ID: &'static str = "window.cursor";
}

const _: () = assert!(core::mem::size_of::<WindowCursor>() == 20);
