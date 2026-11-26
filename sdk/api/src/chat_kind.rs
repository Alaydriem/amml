/// The type of a chat line, as the game gives it: the raw value of the message type of the game. The loader does not name
/// the values, because they belong to the game and may change with a build.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct ChatKind(pub u32);
