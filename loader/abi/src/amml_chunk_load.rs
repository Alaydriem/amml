/// How much of a chunk the client has. A plain integer so that a value from the other side of the boundary can never
/// be invalid.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AmmlChunkLoad(pub u8);

impl AmmlChunkLoad {
    /// The chunk is not fully loaded; nothing past its load state is read.
    pub const NOT_LOADED: Self = Self(0);
    /// The chunk is loaded, but its top non-empty sub-chunk is still the client-request placeholder.
    pub const TOP_PLACEHOLDER: Self = Self(1);
    /// The chunk is loaded and its blocks can be read.
    pub const LOADED: Self = Self(2);
}
