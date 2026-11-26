use crate::LoaderState;

/// The member offsets the UI input adapter reads.
///
/// `controller`, `button_ids`, `pointer` and `input_mode` are offsets in `ScreenView`: the controller pointer, the
/// two pointer button ids (`std::array<uint, 2>`), the last pointer position (two f32) and the input mode (a 32-bit
/// enum, absent when the entry did not resolve). `scene_view` is the offset in `UIScene` of its `ScreenView` pointer.
pub struct UiInputLayout {
    pub controller: usize,
    pub button_ids: usize,
    pub pointer: usize,
    pub input_mode: Option<usize>,
    pub scene_view: usize,
}

impl UiInputLayout {
    /// The layout from the resolved entries; `None` when a needed one is missing.
    pub fn new(state: &LoaderState) -> Option<Self> {
        let value = |name: &str| state.resolve(name).ok().and_then(|value| usize::try_from(value).ok());
        Some(Self {
            controller: value("ScreenView.mController")?,
            button_ids: value("ScreenView.mPointerButtonIds")?,
            pointer: value("ScreenView.mPointerLocationPrevious")?,
            input_mode: value("ScreenView.mInputMode"),
            scene_view: value("UIScene.mScreenView")?,
        })
    }
}
