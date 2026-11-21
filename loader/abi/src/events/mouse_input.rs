use crate::{AmmlEvent, AmmlEventHeader, AmmlMouseAction, AmmlMouseButton};

/// A mouse move, button or wheel event. A listener may cancel it.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct MouseInput {
    pub header: AmmlEventHeader,
    pub action: AmmlMouseAction,
    pub button: AmmlMouseButton,
    pub down: u8,
    pub _reserved: u8,
    pub x: i32,
    pub y: i32,
    pub dx: i32,
    pub dy: i32,
    pub notches: f32,
}

impl AmmlEvent for MouseInput {
    const ID: &'static str = "input.mouse";
    const CANCELLABLE: bool = true;
}

const _: () = assert!(core::mem::size_of::<MouseInput>() == 40);
