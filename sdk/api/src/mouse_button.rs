use amml_abi::AmmlMouseButton;

/// A mouse button.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    X1,
    X2,
}

impl MouseButton {
    /// The button of an ABI value; `None` for a value this crate does not know.
    pub fn from_abi(button: AmmlMouseButton) -> Option<Self> {
        match button {
            AmmlMouseButton::LEFT => Some(Self::Left),
            AmmlMouseButton::RIGHT => Some(Self::Right),
            AmmlMouseButton::MIDDLE => Some(Self::Middle),
            AmmlMouseButton::X1 => Some(Self::X1),
            AmmlMouseButton::X2 => Some(Self::X2),
            _ => None,
        }
    }
}

impl From<MouseButton> for AmmlMouseButton {
    fn from(button: MouseButton) -> Self {
        match button {
            MouseButton::Left => Self::LEFT,
            MouseButton::Right => Self::RIGHT,
            MouseButton::Middle => Self::MIDDLE,
            MouseButton::X1 => Self::X1,
            MouseButton::X2 => Self::X2,
        }
    }
}
