use crate::{AmmlEvent, AmmlEventHeader};

/// A pointer button of a UI screen went down or up (`input.ui_pointer`, Before the game handles it). `screen` is
/// the id of the screen controller of the view, the id of `ScreenRender`;
/// `view` is the id of the view that gets the event. `pointer` is the index of the button in the game's pointer
/// button list: 0 the primary button, 1 the second. `pressed` is 1 for a down and 0 for an up. `input_mode` is an
/// `AmmlInputMode` value (0 when the loader cannot read it). `x` and `y` are the last pointer position of the view,
/// in GUI units.
///
/// A listener may cancel it: the game then does not get this button event. The up of a cancelled down is raised
/// already cancelled and the game does not get it; a cancel of an up whose down reached the game is ignored, so the
/// game never keeps a button down. A button id that is not a pointer button goes to the game with no event.
/// Every view of the screen stack gets the button event, so a click is raised once for each view.
///
/// Raised on the thread that runs the game's handler.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct UiPointer {
    pub header: AmmlEventHeader,
    pub screen: u64,
    pub view: u64,
    pub pointer: u8,
    pub pressed: u8,
    pub input_mode: u8,
    pub _reserved: u8,
    pub x: f32,
    pub y: f32,
    pub _reserved2: u32,
}

impl AmmlEvent for UiPointer {
    const ID: &'static str = "input.ui_pointer";
    const CANCELLABLE: bool = true;
}

const _: () = assert!(core::mem::size_of::<UiPointer>() == 48);
