use amml_abi::AmmlGameType;

/// The game mode of a player. A value the enum does not name keeps the game's raw number in `Other`, so a mode that a
/// later game adds is not lost.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum GameType {
    Survival,
    Creative,
    Adventure,
    Spectator,
    /// The level's default mode: the game's raw value 5, kept when the player has no mode of its own.
    Default,
    Other(i32),
}

impl GameType {
    /// The game's raw value.
    pub fn raw(self) -> i32 {
        match self {
            Self::Survival => AmmlGameType::SURVIVAL.0,
            Self::Creative => AmmlGameType::CREATIVE.0,
            Self::Adventure => AmmlGameType::ADVENTURE.0,
            Self::Spectator => AmmlGameType::SPECTATOR.0,
            Self::Default => AmmlGameType::DEFAULT.0,
            Self::Other(value) => value,
        }
    }
}

impl From<AmmlGameType> for GameType {
    fn from(value: AmmlGameType) -> Self {
        match value {
            AmmlGameType::SURVIVAL => Self::Survival,
            AmmlGameType::CREATIVE => Self::Creative,
            AmmlGameType::ADVENTURE => Self::Adventure,
            AmmlGameType::SPECTATOR => Self::Spectator,
            AmmlGameType::DEFAULT => Self::Default,
            AmmlGameType(other) => Self::Other(other),
        }
    }
}

impl From<GameType> for AmmlGameType {
    fn from(value: GameType) -> Self {
        AmmlGameType(value.raw())
    }
}
