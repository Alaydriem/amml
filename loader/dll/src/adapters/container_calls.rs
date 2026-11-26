use crate::LoaderState;

/// The derived values `ContainerAdapter` calls the game's container screen with: the addresses of the four
/// handlers that are plain functions, the vtable slots of the visual stack read and of `_handlePlaceAll`, the offset
/// of the container manager controller in the screen controller and the slot of `handleSwap` in the manager.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ContainerCalls {
    pub auto_place: usize,
    pub take_all: usize,
    pub drop_item: usize,
    pub drop_selected: usize,
    pub place_all_slot: usize,
    pub visual_slot: usize,
    pub manager: usize,
    pub swap_slot: usize,
}

impl ContainerCalls {
    /// The calls from the resolved entries; `None` when one is missing.
    pub fn new(state: &LoaderState) -> Option<Self> {
        let value = |name: &str| state.resolve(name).ok().and_then(|value| usize::try_from(value).ok());
        Some(Self {
            auto_place: value("ContainerScreenController._handleAutoPlace").filter(|value| *value != 0)?,
            take_all: value("ContainerScreenController._handleTakeAll").filter(|value| *value != 0)?,
            drop_item: value("ContainerScreenController._handleDropItem").filter(|value| *value != 0)?,
            drop_selected: value("ContainerScreenController._handleDropSelectedItem").filter(|value| *value != 0)?,
            place_all_slot: value("ContainerScreenController._handlePlaceAll")?,
            visual_slot: value("ContainerScreenController.getVisualItemStackImpl")?,
            // A manager offset of 0 would read the controller's vtable as the manager pointer.
            manager: value("ContainerScreenController.mContainerManagerController").filter(|value| *value != 0)?,
            swap_slot: value("ContainerManagerController.handleSwap")?,
        })
    }
}
