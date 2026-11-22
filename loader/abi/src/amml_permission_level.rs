/// The player permission level as the game's raw value: 0 visitor, 1 member, 2 operator, 3 custom.
///
/// Only 2 (operator) and 3 (custom) are read from the executable (the shared loader plan's actor task). The names of 0
/// (visitor) and 1 (member) follow the game's `PlayerPermissionLevel` order as the Bedrock protocol documents it; no
/// exe reading proves them, so a mod that compares with `VISITOR` or `MEMBER` relies on that order.
///
/// A plain integer so that a value the game adds later crosses the boundary unchanged.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct AmmlPermissionLevel(pub u8);

impl AmmlPermissionLevel {
    /// Not read from the executable (see the type's doc).
    pub const VISITOR: Self = Self(0);
    /// Not read from the executable (see the type's doc).
    pub const MEMBER: Self = Self(1);
    pub const OPERATOR: Self = Self(2);
    pub const CUSTOM: Self = Self(3);
}
