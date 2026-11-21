use crate::ChatKind;

/// One chat line as the game built it, for `FakeGame::raise_chat`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FakeChat {
    /// The type of the line.
    pub kind: ChatKind,
    /// The name of the sender; empty for a line with no sender.
    pub username: String,
    /// The Xbox user id of the author; empty when the line has none.
    pub xuid: String,
    /// The whole line as the game shows it.
    pub text: String,
    /// The whole line for a player who filters chat; `None` when the line has none.
    pub filtered_text: Option<String>,
}
