use crate::{AmmlEvent, AmmlEventHeader};

/// The view turns by `yaw` and `pitch`. A listener may cancel it.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct LookDelta {
    pub header: AmmlEventHeader,
    pub yaw: f32,
    pub pitch: f32,
}

impl AmmlEvent for LookDelta {
    const ID: &'static str = "input.look_delta";
    const CANCELLABLE: bool = true;
}

const _: () = assert!(core::mem::size_of::<LookDelta>() == 24);
