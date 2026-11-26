/// The 16 bytes of a player's `mce::UUID`, as the game keeps them in memory. Two uuids are the same player when the bytes
/// are equal.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Uuid(pub [u8; 16]);
