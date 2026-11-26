use crate::{AmmlEvent, AmmlEventHeader};

/// The game window gained or lost the focus. A listener may cancel a loss only: the game then does not see it
/// until a mod replays it, and the replay comes with `replayed = 1`. A cancel of a gain or of a replay is
/// ignored.
///
/// Raised on thread `WINDOW`, or `GAME` when the window thread is the game thread.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct WindowFocus {
    pub header: AmmlEventHeader,
    pub gained: u8,
    pub replayed: u8,
    pub _reserved: [u8; 2],
}

impl AmmlEvent for WindowFocus {
    const ID: &'static str = "window.focus";
    const CANCELLABLE: bool = true;
}

const _: () = assert!(core::mem::size_of::<WindowFocus>() == 20);
