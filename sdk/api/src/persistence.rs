use amml_abi::AmmlPersistence;

/// How long a value that `Options::set_int` writes lasts.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Persistence {
    /// The game is told not to save the value. The loader keeps the value from before the first transient set of the
    /// key and puts it back (and saves it) at `Options::release_transient` or when this mod is disabled, but only while
    /// the option still holds the value this mod set: a value the user changed meanwhile stays.
    Transient,
}

impl Persistence {
    pub(crate) fn to_abi(self) -> AmmlPersistence {
        match self {
            Self::Transient => AmmlPersistence::TRANSIENT,
        }
    }
}
