/// Which folder `AmmlCoreTable::dir` gives.
///
/// A plain integer so that a value from the other side of the boundary can never be invalid.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AmmlDirKind(pub u8);

impl AmmlDirKind {
    pub const MOD: Self = Self(0);
    pub const CONFIG: Self = Self(1);
    pub const DATA: Self = Self(2);
    pub const SHARED_DATA: Self = Self(3);
    pub const INSTANCE: Self = Self(4);
    pub const GAME_DATA: Self = Self(5);
}
