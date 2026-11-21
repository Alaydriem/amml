use crate::{AmmlEvent, AmmlEventHeader};

/// The game grabs the mouse. A listener may cancel it.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct MouseGrab {
    pub header: AmmlEventHeader,
}

impl AmmlEvent for MouseGrab {
    const ID: &'static str = "client.mouse_grab";
    const CANCELLABLE: bool = true;
}

const _: () = assert!(core::mem::size_of::<MouseGrab>() == 16);
