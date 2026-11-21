use crate::LoaderState;

/// The member offsets and virtual slots the reads of one screen view use.
///
/// `controller`, `size`, `pointer` and `input_mode` are offsets in `ScreenView`. `container_slot` is the slot of
/// `ScreenController::_isContainerScreen` in the controller vtable, `options_slot` the slot of
/// `ClientInstance::getOptions` and `ui_profile_slot` the slot of `IOptionRegistry::getUIProfile`.
pub struct ScreenLayout {
    pub controller: usize,
    pub size: usize,
    pub pointer: usize,
    pub input_mode: usize,
    pub container_slot: usize,
    pub options_slot: usize,
    pub ui_profile_slot: usize,
}

impl ScreenLayout {
    /// The layout from the resolved entries; `None` when one is missing.
    pub fn new(state: &LoaderState) -> Option<Self> {
        let value = |name: &str| state.resolve(name).ok().and_then(|value| usize::try_from(value).ok());
        Some(Self {
            controller: value("ScreenView.mController")?,
            size: value("ScreenView.mSize")?,
            pointer: value("ScreenView.mPointerLocationPrevious")?,
            input_mode: value("ScreenView.mInputMode")?,
            container_slot: value("ScreenController.isContainerScreen")?,
            options_slot: value("ClientInstance.getOptions")?,
            ui_profile_slot: value("IOptionRegistry.getUIProfile")?,
        })
    }
}
