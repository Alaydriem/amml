use amml_abi::{AmmlMouseAction, MouseInput};

use crate::{MouseAction, MouseButton};

/// Typed reads of a `MouseInput`.
pub trait MouseInputExt {
    fn action(&self) -> MouseAction;
}

impl MouseInputExt for MouseInput {
    fn action(&self) -> MouseAction {
        match self.action {
            AmmlMouseAction::MOVE => MouseAction::Move,
            AmmlMouseAction::BUTTON => match MouseButton::from_abi(self.button) {
                Some(button) => MouseAction::Button { button, down: self.down != 0 },
                None => MouseAction::Other,
            },
            AmmlMouseAction::WHEEL => MouseAction::Wheel { notches: self.notches },
            _ => MouseAction::Other,
        }
    }
}
