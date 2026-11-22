use amml_abi::AmmlOptionKey;

/// An integer game option that `Options::get_int` and `set_int` reach.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct OptionKey(AmmlOptionKey);

impl OptionKey {
    /// Vertical sync: 0 off, 1 on, 2 adaptive.
    pub const VSYNC: Self = Self(AmmlOptionKey::VSYNC);
    /// The maximum frame rate the game uses now; 0 is no limit.
    pub const MAX_FRAMERATE: Self = Self(AmmlOptionKey::MAX_FRAMERATE);

    pub(crate) fn to_abi(self) -> AmmlOptionKey {
        self.0
    }

    // The name of the key in the fake loader's write log.
    #[cfg(feature = "test")]
    pub(crate) fn name(self) -> &'static str {
        if self == Self::VSYNC { "vsync" } else { "max_framerate" }
    }

    // The key for a value of the ABI, for the fake loader.
    #[cfg(feature = "test")]
    pub(crate) fn from_abi(key: AmmlOptionKey) -> Option<Self> {
        [Self::VSYNC, Self::MAX_FRAMERATE].into_iter().find(|known| known.0 == key)
    }
}
