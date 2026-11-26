/// The game mode of a player as the game's raw value: 0 survival, 1 creative, 2 adventure, 5 default, 6 spectator.
///
/// A plain integer so that a value the game adds later crosses the boundary unchanged.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct AmmlGameType(pub i32);

impl AmmlGameType {
    pub const SURVIVAL: Self = Self(0);
    pub const CREATIVE: Self = Self(1);
    pub const ADVENTURE: Self = Self(2);
    pub const DEFAULT: Self = Self(5);
    pub const SPECTATOR: Self = Self(6);
}
