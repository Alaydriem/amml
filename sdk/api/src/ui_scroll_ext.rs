use amml_abi::UiScroll;

use crate::ScreenId;

/// Typed reads of a `UiScroll`. Cancel it with `Cancel::cancel`: the scene does not get the step.
pub trait UiScrollExt {
    /// The screen of the scene's view; `None` when the loader does not know it.
    fn screen(&self) -> Option<ScreenId>;

    /// The step: positive away from the user, negative toward the user. One wheel notch is one event.
    fn notches(&self) -> i32;
}

impl UiScrollExt for UiScroll {
    fn screen(&self) -> Option<ScreenId> {
        (self.screen != 0).then_some(ScreenId(self.screen))
    }

    fn notches(&self) -> i32 {
        self.notches
    }
}
