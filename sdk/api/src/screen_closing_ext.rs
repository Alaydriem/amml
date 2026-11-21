use amml_abi::ScreenClosing;

use crate::ScreenId;

/// Typed reads of a `ScreenClosing`.
pub trait ScreenClosingExt {
    /// The screen the game is about to free. Its id is never used again.
    fn screen(&self) -> ScreenId;
}

impl ScreenClosingExt for ScreenClosing {
    fn screen(&self) -> ScreenId {
        ScreenId(self.screen)
    }
}
