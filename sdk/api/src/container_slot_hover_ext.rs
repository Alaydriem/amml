use amml_abi::{ContainerSlotHover, ContainerSlotUnhover};

use crate::{ScreenId, SlotAt};

/// Typed reads of a `ContainerSlotHover` and a `ContainerSlotUnhover`.
pub trait ContainerSlotHoverExt {
    /// The screen the pointer is over, the id of `ScreenRender`.
    fn screen(&self) -> ScreenId;

    /// The slot the pointer moved onto (hover) or off (unhover); its collection is `Other` for a collection this API
    /// does not name.
    fn slot(&self) -> SlotAt;
}

impl ContainerSlotHoverExt for ContainerSlotHover {
    fn screen(&self) -> ScreenId {
        ScreenId(self.screen)
    }

    fn slot(&self) -> SlotAt {
        SlotAt::from(self.slot)
    }
}

impl ContainerSlotHoverExt for ContainerSlotUnhover {
    fn screen(&self) -> ScreenId {
        ScreenId(self.screen)
    }

    fn slot(&self) -> SlotAt {
        SlotAt::from(self.slot)
    }
}
