use crate::{AmmlEvent, AmmlEventHeader, AmmlSlotAt};

/// The pointer left a slot of a container screen (`ui.container_slot_unhover`, After the game handled the
/// unhover). `screen` and `slot` are as for `ContainerSlotHover`.
///
/// Raised on the thread that runs the game's unhover handler.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ContainerSlotUnhover {
    pub header: AmmlEventHeader,
    pub screen: u64,
    pub slot: AmmlSlotAt,
}

impl AmmlEvent for ContainerSlotUnhover {
    const ID: &'static str = "ui.container_slot_unhover";
}

const _: () = assert!(core::mem::size_of::<ContainerSlotUnhover>() == 32);
