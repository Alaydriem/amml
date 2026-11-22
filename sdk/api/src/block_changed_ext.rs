use amml_abi::BlockChanged;

use crate::{BlockHandle, BlockPos, BlockTypeId, Ctx};

/// Typed reads of a `BlockChanged`: a block of the dimension of the local player changed, and the event comes after
/// the game's own change.
///
/// The event is raised at once when the game changed the block on the game thread, else at the next `FrameUpdate`
/// After. A change from a state of one type to another state of the same type has `new` equal to `old` and is raised
/// too: compare the types or read the block again. The block types are ids of `Blocks::type_name`.
pub trait BlockChangedExt {
    /// The id of the dimension whose block source changed (the id `Dimension::id` gives).
    fn dimension(&self) -> i32;

    /// The block position that changed.
    fn pos(&self) -> BlockPos;

    /// The block layer: 0 the main layer, 1 the liquid layer.
    fn layer(&self) -> u32;

    /// The type of the block after the change; `BlockTypeId::NONE` when the game gave no readable type. Named `new` as
    /// the pair of `old`; it is a read, not a constructor.
    #[allow(clippy::new_ret_no_self, clippy::wrong_self_convention)]
    fn new(&self) -> BlockTypeId;

    /// The type of the block before the change; `BlockTypeId::NONE` when the game gave no readable type.
    fn old(&self) -> BlockTypeId;

    /// The block after the change, as an owned `BlockHandle` of this mod (version 2 of the event). Valid only inside the
    /// listener call that got the event: a copy of the event kept past it gives `None`, as do an event of version 1, a
    /// change with no block and a refused read. The handle itself stays valid in this world.
    fn new_state(&self, ctx: &Ctx) -> Option<BlockHandle>;
}

impl BlockChangedExt for BlockChanged {
    fn dimension(&self) -> i32 {
        self.dimension
    }

    fn pos(&self) -> BlockPos {
        BlockPos::from(self.pos)
    }

    fn layer(&self) -> u32 {
        self.layer
    }

    #[allow(clippy::new_ret_no_self, clippy::wrong_self_convention)]
    fn new(&self) -> BlockTypeId {
        BlockTypeId(self.new)
    }

    fn old(&self) -> BlockTypeId {
        BlockTypeId(self.old)
    }

    fn new_state(&self, ctx: &Ctx) -> Option<BlockHandle> {
        if self.header.version < 2 || self.new_state == 0 {
            return None;
        }
        ctx.blocks().changed_state(self.new_state)
    }
}
