/// The UI profile option of the game: the layout family of its screens.
///
/// A plain integer so that a value from the other side of the boundary can never be invalid. The game
/// reports 0 or 1 for the option; the names come from the game's headers.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct AmmlUiProfile(pub u8);

impl AmmlUiProfile {
    pub const CLASSIC: Self = Self(0);
    pub const POCKET: Self = Self(1);
}
