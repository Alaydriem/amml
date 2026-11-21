/// The id the loader gives a block type (`minecraft:stone`): the same number for the same type for the rest of the
/// process, never given to another type. 0 is no block type. Ask `Blocks::type_name` for its name.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub struct BlockTypeId(pub u32);

impl BlockTypeId {
    /// No block type.
    pub const NONE: Self = Self(0);

    /// Whether this is `NONE`.
    pub fn is_none(&self) -> bool {
        *self == Self::NONE
    }
}
