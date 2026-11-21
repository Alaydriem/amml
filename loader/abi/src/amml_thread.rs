/// The game thread an event is delivered on.
///
/// A plain integer so that a value from the other side of the boundary can never be invalid.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AmmlThread(pub u8);

impl AmmlThread {
    pub const GAME: Self = Self(0);
    pub const RENDER: Self = Self(1);
    pub const NETWORK: Self = Self(2);
    pub const SERVER: Self = Self(3);
    /// The thread of the game window's message loop, when it is not the game thread.
    pub const WINDOW: Self = Self(4);
    /// An input feed that is not the game thread.
    pub const INPUT: Self = Self(5);
}
