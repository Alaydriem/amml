/// A string of the chat line of a running `ChatMessageAdded`.
///
/// A plain integer so that a value from the other side of the boundary can never be invalid; a loader refuses a value
/// it does not know.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AmmlChatField(pub u8);

impl AmmlChatField {
    /// The name of the sender; empty for a line with no sender.
    pub const USERNAME: Self = Self(1);
    /// The Xbox user id of the author; empty when the line has none.
    pub const XUID: Self = Self(2);
    /// The whole line as the game shows it (`<name> message`, or the message alone).
    pub const TEXT: Self = Self(3);
    /// The whole line the game shows when the player filters chat. A line with no filtered text gives `NOT_AVAILABLE`.
    pub const FILTERED_TEXT: Self = Self(4);
}
