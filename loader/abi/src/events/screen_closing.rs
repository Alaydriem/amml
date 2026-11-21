use crate::{AmmlEvent, AmmlEventHeader};

/// The game is about to free the screen controller `screen` (`ui.screen_closing`, Before). Raised only for a
/// screen id the loader gave out. The id is never used again.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ScreenClosing {
    pub header: AmmlEventHeader,
    pub screen: u64,
}

impl AmmlEvent for ScreenClosing {
    const ID: &'static str = "ui.screen_closing";
}

const _: () = assert!(core::mem::size_of::<ScreenClosing>() == 24);
