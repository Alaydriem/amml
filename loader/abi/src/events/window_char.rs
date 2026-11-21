use crate::{AmmlEvent, AmmlEventHeader};

/// A character typed into the game window, as a Unicode scalar value. A listener may cancel it.
///
/// Raised on thread `WINDOW`, or `GAME` when the window thread is the game thread.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct WindowChar {
    pub header: AmmlEventHeader,
    pub ch: u32,
}

impl AmmlEvent for WindowChar {
    const ID: &'static str = "window.char";
    const CANCELLABLE: bool = true;
}

const _: () = assert!(core::mem::size_of::<WindowChar>() == 20);
