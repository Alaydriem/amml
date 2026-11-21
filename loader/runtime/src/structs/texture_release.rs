/// A game texture reference the pump gives back.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextureRelease {
    /// The address of the held `TexturePtr`; 0 when the entry held none.
    pub held: usize,
    /// The name to unload from the client texture group, for an uploaded texture.
    pub unload_name: Option<String>,
}
