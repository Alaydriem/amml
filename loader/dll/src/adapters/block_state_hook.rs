/// The calls of the `world.block_states` area that note the thread they run on, one note each
/// (`BlockStateAdapter`).
#[derive(Clone, Copy)]
pub enum BlockStateHook {
    FromState,
    Rotated,
    Name,
    Item,
}

impl BlockStateHook {
    /// The game entry the call stands for, as the `world thread` line names it.
    pub fn entry(self) -> &'static str {
        match self {
            Self::FromState => "BlockSerializationUtils.tryGetBlockFromNBT",
            Self::Rotated => "VanillaBlockStateTransformUtils.transformBlock",
            Self::Name => "BlockType.mNameInfo.mFullName",
            Self::Item => "BlockType.asItemInstance",
        }
    }
}
