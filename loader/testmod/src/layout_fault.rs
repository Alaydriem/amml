/// Why a name-tag list does not have the layout the mod expects.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LayoutFault {
    /// The element size entry is 0.
    ZeroElementSize,
    /// The pointers do not satisfy first <= last <= end, or first is null while last is not.
    Unordered,
    /// The bytes between first and last are not a whole number of elements.
    Misaligned,
}

impl LayoutFault {
    /// The short name used in log lines.
    pub fn name(self) -> &'static str {
        match self {
            Self::ZeroElementSize => "element size is 0",
            Self::Unordered => "pointers are not ordered",
            Self::Misaligned => "length is not a whole number of elements",
        }
    }
}
