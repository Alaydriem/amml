/// How a request of `request_moves` ended, as `InventoryRequestDone` gives it.
///
/// A plain integer so that a value from the other side of the boundary can never be invalid.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct AmmlRequestOutcome(pub u8);

impl AmmlRequestOutcome {
    /// The server accepted every move.
    pub const DONE: Self = Self(1);
    /// The server refused a move.
    pub const REFUSED: Self = Self(2);
    /// The server did not answer every move in time.
    pub const NO_ANSWER: Self = Self(3);
    /// The server did not open the inventory in time (`hidden_open` only).
    pub const NO_OPEN: Self = Self(4);
    /// A screen opened before the moves were sent (or, with the hidden open, after them), or the world was left.
    pub const CANCELLED: Self = Self(5);
    /// The slots did not hold what a move needs when it was to be sent; nothing was sent.
    pub const STALE: Self = Self(6);
    /// The loader could not send a packet of the request (the game made no packet, or refused its payload); nothing was
    /// predicted.
    pub const NOT_SENT: Self = Self(7);
}
