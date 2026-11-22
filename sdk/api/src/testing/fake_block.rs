use crate::testing::FakeItem;
use crate::{BlockHandle, BlockTypeId};

/// One block of a `FakeWorld` (`FakeWorld::blocks`): what `BlockSource::block` and the `Block` reads give for its
/// position. `name` is what `Block::name` gives for `type_id`, as `Blocks::type_name` would (the fake world's `names`
/// win when both name the type).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FakeBlock {
    pub type_id: BlockTypeId,
    pub name: &'static str,
    pub is_air: bool,
    /// What `Block::can_be_built_over` gives.
    pub built_over: bool,
    /// What `Block::is_interactive` gives for a block that is not air.
    pub interactive: bool,
    /// What `Block::item` gives; `None` is a block with no item (the call is refused).
    pub item: Option<FakeItem>,
    /// What `Block::handle` gives; `None` is a refused read.
    pub handle: Option<BlockHandle>,
}
