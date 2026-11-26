/// How long a value that `AmmlOptionsTable::set_int` writes lasts.
///
/// A plain integer so that a value from the other side of the boundary can never be invalid. Zero is not a value.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AmmlPersistence(pub u8);

impl AmmlPersistence {
    /// The value is not saved to the options file. The loader keeps the value from before the first transient set of
    /// the key and writes it back (and saves it) at `release_transient` or at the owner's disable.
    pub const TRANSIENT: Self = Self(1);
}
