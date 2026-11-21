/// A move of a container screen, as the game's own buttons make it.
///
/// A plain integer so that a value from the other side of the boundary can never be invalid; the loader gives
/// `INVALID_ARGUMENT` for an action it does not know.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct AmmlContainerAction(pub u8);

impl AmmlContainerAction {
    /// The whole stack of the slot goes to the other side (shift-click).
    pub const AUTO_PLACE: Self = Self(1);
    /// The whole stack of the slot goes onto the cursor.
    pub const TAKE_ALL: Self = Self(2);
    /// The cursor stack goes into the slot.
    pub const PLACE_ALL: Self = Self(3);
    /// The stack of the slot is dropped out of the screen.
    pub const DROP_SLOT: Self = Self(4);
    /// The cursor stack is dropped out of the screen.
    pub const DROP_CURSOR: Self = Self(5);
}
