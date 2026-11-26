use amml_abi::WindowChanged;

use crate::WindowChange;

/// Typed reads of a `WindowChanged`.
pub trait WindowChangedExt {
    fn change(&self) -> WindowChange;
}

impl WindowChangedExt for WindowChanged {
    fn change(&self) -> WindowChange {
        WindowChange::from(self.kind)
    }
}
