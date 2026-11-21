use crate::{AmmlEvent, AmmlEventHeader};

/// The game builds the hover text of an item (`ui.item_hovertext`, Before). `kind` says what item it is.
/// `contents_lines` starts at 1 and is the only field a listener can change: when a listener leaves it 0, the
/// loader builds the text without the contents lines of the item (the name and enchantment lines stay); any other
/// value gives the game's own text. The loader writes `kind` and `_reserved` back after every listener.
///
/// Version 1 is raised for the shulker box items only.
///
/// Raised on the thread that runs the game's handler.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ItemHovertext {
    pub header: AmmlEventHeader,
    pub kind: u32,
    pub contents_lines: u8,
    pub _reserved: [u8; 3],
}

impl ItemHovertext {
    /// An item that holds other items (a shulker box).
    pub const CONTAINER_ITEM: u32 = 1;
}

impl AmmlEvent for ItemHovertext {
    const ID: &'static str = "ui.item_hovertext";
}

const _: () = assert!(core::mem::size_of::<ItemHovertext>() == 24);
