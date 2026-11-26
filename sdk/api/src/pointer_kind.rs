use amml_abi::AmmlPointerKind;

/// What a `WindowPointer` reports. A value this crate does not know is `Other`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PointerKind {
    /// The pointer moved inside the client area.
    Move,
    /// A mouse button went down or up.
    Button,
    /// The wheel turned.
    Wheel,
    /// The pointer left the client area.
    Leave,
    Other,
}

impl From<AmmlPointerKind> for PointerKind {
    fn from(kind: AmmlPointerKind) -> Self {
        match kind {
            AmmlPointerKind::MOVE => Self::Move,
            AmmlPointerKind::BUTTON => Self::Button,
            AmmlPointerKind::WHEEL => Self::Wheel,
            AmmlPointerKind::LEAVE => Self::Leave,
            _ => Self::Other,
        }
    }
}
