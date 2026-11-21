use crate::{AmmlEvent, AmmlEventHeader, AmmlScreen};

/// The screen on top changed (`ui.screen_changed`, raised at `ClientTick`). `old` and `new` are screen ids, the
/// ids of `ScreenRender` and `ScreenClosing`: the top screen is the controller of the last view drawn since the
/// previous tick, so the event comes one frame after the change. `new` is 0 when no view was drawn. `screen` is
/// the game's top screen name as a `client.top_screen` value. A notice drawn over a screen (`TOAST`) is no change.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ScreenChanged {
    pub header: AmmlEventHeader,
    pub old: u64,
    pub new: u64,
    pub screen: AmmlScreen,
    pub _reserved: [u8; 7],
}

impl AmmlEvent for ScreenChanged {
    const ID: &'static str = "ui.screen_changed";
}

const _: () = assert!(core::mem::size_of::<ScreenChanged>() == 40);
