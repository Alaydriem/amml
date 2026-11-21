use amml_abi::ItemHovertext;

/// What item an `ItemHovertext` is for.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum HovertextKind {
    /// An item that holds other items (a shulker box).
    ContainerItem,
    /// A kind this crate does not name.
    Other(u32),
}

impl From<u32> for HovertextKind {
    fn from(kind: u32) -> Self {
        match kind {
            ItemHovertext::CONTAINER_ITEM => Self::ContainerItem,
            other => Self::Other(other),
        }
    }
}
