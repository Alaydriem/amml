use amml_data::AdapterCatalog;

/// How the loader gives the pick range on this build: the catalog shape of `player.pick_range` that resolved.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PickRangeShape {
    /// `GameMode::getMaxPickRange` on the player's game mode: the range in `xmm0` (26.63).
    Getter,
    /// `Player::isCreative` picks one of the two range constants the game loads after it (70.x, where the game inlines
    /// `getMaxPickRange`).
    Creative,
}

impl PickRangeShape {
    /// The range of a creative player: the constant 70.26 loads when `Player::isCreative` gave true (`movss
    /// xmm0,[12E8CE44h]` at 0x387065a, bytes `00 00 40 41`); 26.63 `getMaxPickRange` returns the same value
    /// (0xF085AA4).
    pub const CREATIVE_RANGE: f32 = 12.0;
    /// The range of any other player: the constant 70.26 loads when `Player::isCreative` gave false (`movss
    /// xmm0,[12E8CE48h]` at 0x3870650, bytes `66 66 D6 40`); 26.63 `getMaxPickRange` returns the same value
    /// (0xF133370).
    pub const OTHER_RANGE: f32 = f32::from_bits(0x40D6_6666);

    /// The shape that goes with the catalog shape name `name`; `None` for a name the catalog does not give.
    pub fn from_catalog(name: &str) -> Option<Self> {
        match name {
            AdapterCatalog::PICK_RANGE_GETTER_SHAPE => Some(Self::Getter),
            AdapterCatalog::PICK_RANGE_CREATIVE_SHAPE => Some(Self::Creative),
            _ => None,
        }
    }

    /// The catalog shape name, for the log line.
    pub fn name(self) -> &'static str {
        match self {
            Self::Getter => AdapterCatalog::PICK_RANGE_GETTER_SHAPE,
            Self::Creative => AdapterCatalog::PICK_RANGE_CREATIVE_SHAPE,
        }
    }

    /// The range for the result of `Player::isCreative`.
    pub fn range_of(creative: bool) -> f32 {
        if creative { Self::CREATIVE_RANGE } else { Self::OTHER_RANGE }
    }
}
