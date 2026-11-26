/// Where a listener runs among the listeners of one event. Lower values run first.
///
/// A plain integer so that a value from the other side of the boundary can never be invalid.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct AmmlBand(pub u8);

impl AmmlBand {
    pub const FIRST: Self = Self(0);
    pub const WRITE: Self = Self(1);
    pub const NORMAL: Self = Self(2);
    pub const READ_AFTER_WRITERS: Self = Self(3);
    pub const LAST: Self = Self(4);
}
