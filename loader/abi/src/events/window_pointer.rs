use crate::{AmmlEvent, AmmlEventHeader, AmmlModifiers, AmmlMouseButton, AmmlPoint, AmmlPointerKind};

/// The pointer moved, a button went down or up, the wheel turned or the pointer left the client area, as the
/// game window saw it. `client` is in client coordinates, `buffer` in back buffer coordinates. A listener may
/// cancel it; the game then does not see the message.
///
/// Raised on thread `WINDOW`, or `GAME` when the window thread is the game thread.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct WindowPointer {
    pub header: AmmlEventHeader,
    pub kind: AmmlPointerKind,
    pub button: AmmlMouseButton,
    pub down: u8,
    pub _reserved: u8,
    pub modifiers: AmmlModifiers,
    pub _reserved2: u16,
    pub client: AmmlPoint,
    pub buffer: AmmlPoint,
    pub notches: f32,
}

impl AmmlEvent for WindowPointer {
    const ID: &'static str = "window.pointer";
    const CANCELLABLE: bool = true;
}

const _: () = assert!(core::mem::size_of::<WindowPointer>() == 44);
