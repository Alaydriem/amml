use amml_abi::ScreenChanged;

use crate::{Screen, ScreenId};

/// Typed reads of a `ScreenChanged`.
pub trait ScreenChangedExt {
    /// The screen that was on top before; `ScreenId::NONE` before the first change.
    fn old(&self) -> ScreenId;

    /// The screen on top now; `ScreenId::NONE` when no view was drawn. The change comes one frame after it
    /// happened, and a toast over a screen is no change.
    // `new` is the field name of the event, as `old` is.
    #[allow(clippy::wrong_self_convention, clippy::new_ret_no_self)]
    fn new(&self) -> ScreenId;

    /// The top screen name of the game as `Client::top_screen` reads it.
    fn screen(&self) -> Screen;
}

impl ScreenChangedExt for ScreenChanged {
    fn old(&self) -> ScreenId {
        ScreenId(self.old)
    }

    #[allow(clippy::wrong_self_convention, clippy::new_ret_no_self)]
    fn new(&self) -> ScreenId {
        ScreenId(self.new)
    }

    fn screen(&self) -> Screen {
        Screen::from(self.screen)
    }
}
