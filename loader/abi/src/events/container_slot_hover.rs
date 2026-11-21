use crate::{AmmlEvent, AmmlEventHeader, AmmlSlotAt};

/// The pointer moved onto a slot of a container screen (`ui.container_slot_hover`, After the game handled the
/// hover). `screen` is the id of the screen controller, the id of `ScreenRender`; `slot` is the slot, with the
/// collection `OTHER` for a collection this API does not name.
///
/// Raised on the thread that runs the game's hover handler. Whether it comes before or after `ScreenRender` in
/// one frame is not promised.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ContainerSlotHover {
    pub header: AmmlEventHeader,
    pub screen: u64,
    pub slot: AmmlSlotAt,
}

impl AmmlEvent for ContainerSlotHover {
    const ID: &'static str = "ui.container_slot_hover";
}

const _: () = assert!(core::mem::size_of::<ContainerSlotHover>() == 32);
