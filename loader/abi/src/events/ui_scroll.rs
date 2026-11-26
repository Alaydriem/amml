use crate::{AmmlEvent, AmmlEventHeader};

/// One wheel step reached the scene of a UI screen (`input.ui_scroll`, Before the scene handles it). One wheel
/// notch is one event. `screen` is the id of the screen controller of the scene's view, or 0 when the loader does
/// not know it. `notches` is the sign of the game's step: positive away from the user, negative toward the user.
///
/// A listener may cancel it: the scene then does not get the step.
///
/// Raised on the thread that runs the game's handler.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct UiScroll {
    pub header: AmmlEventHeader,
    pub screen: u64,
    pub notches: i32,
    pub _reserved: u32,
}

impl AmmlEvent for UiScroll {
    const ID: &'static str = "input.ui_scroll";
    const CANCELLABLE: bool = true;
}

const _: () = assert!(core::mem::size_of::<UiScroll>() == 32);
