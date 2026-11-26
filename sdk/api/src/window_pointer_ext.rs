use amml_abi::{AmmlPointerKind, WindowPointer};

use crate::{Modifiers, MouseButton, Point, PointerKind};

/// Typed reads of a `WindowPointer`.
pub trait WindowPointerExt {
    fn kind(&self) -> PointerKind;
    /// The pointer in client coordinates.
    fn client(&self) -> Point;
    /// The pointer in back buffer coordinates; the same as `client` until the loader knows the back
    /// buffer size.
    fn buffer(&self) -> Point;
    /// The button of a `PointerKind::Button` event; `None` for any other kind (the loader writes `LEFT`
    /// there) and for a button this crate does not know.
    fn button(&self) -> Option<MouseButton>;
    /// Whether the button went down; false for any kind but `Button`.
    fn down(&self) -> bool;
    /// The wheel notches of a `PointerKind::Wheel` event, positive away from the user.
    fn notches(&self) -> f32;
    fn modifiers(&self) -> Modifiers;
}

impl WindowPointerExt for WindowPointer {
    fn kind(&self) -> PointerKind {
        PointerKind::from(self.kind)
    }

    fn client(&self) -> Point {
        Point::from(self.client)
    }

    fn buffer(&self) -> Point {
        Point::from(self.buffer)
    }

    fn button(&self) -> Option<MouseButton> {
        if self.kind != AmmlPointerKind::BUTTON {
            return None;
        }
        MouseButton::from_abi(self.button)
    }

    fn down(&self) -> bool {
        self.kind == AmmlPointerKind::BUTTON && self.down != 0
    }

    fn notches(&self) -> f32 {
        self.notches
    }

    fn modifiers(&self) -> Modifiers {
        Modifiers::from(self.modifiers)
    }
}
