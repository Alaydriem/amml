use amml_abi::{AmmlMouseButton, InjectedInputReleased};

use crate::{Event, KeyCode, MouseButton};

/// Typed reads of an `InjectedInputReleased`.
pub trait InjectedInputReleasedExt {
    /// The one key the loader released, when it released only one (a real key up or a key reset of the
    /// game ended the hold); `None` when it released every input of the lease, and from a loader that sends
    /// version 1 of the event.
    fn released_key(&self) -> Option<KeyCode>;
    /// The one mouse button the loader released, under the rule of `released_key`.
    fn released_button(&self) -> Option<MouseButton>;
}

impl InjectedInputReleasedExt for InjectedInputReleased {
    fn released_key(&self) -> Option<KeyCode> {
        (self.header().version >= 2 && self.input == Self::KEY).then_some(KeyCode(self.code))
    }

    fn released_button(&self) -> Option<MouseButton> {
        if self.header().version < 2 || self.input != Self::BUTTON {
            return None;
        }
        let button = u8::try_from(self.code).ok()?;
        MouseButton::from_abi(AmmlMouseButton(button))
    }
}
