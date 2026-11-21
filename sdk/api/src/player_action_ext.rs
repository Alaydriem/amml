use amml_abi::{BlockBreakContinue, BlockBreakStart, ItemUse, ItemUseOnBlock, PlayerAttack};

use crate::{BlockPos, Face};

/// Typed reads of the five player-action events.
pub trait PlayerActionExt {
    /// Whether the action's player is the local player of the primary client.
    fn is_local_player(&self) -> bool;
}

macro_rules! player_action {
    ($($event:ty),*) => {$(
        impl PlayerActionExt for $event {
            fn is_local_player(&self) -> bool {
                self.is_local_player != 0
            }
        }
    )*};
}

player_action!(BlockBreakStart, BlockBreakContinue, ItemUse, PlayerAttack, ItemUseOnBlock);

/// The block and face of the player-action events on a block.
pub trait BlockActionExt: PlayerActionExt {
    fn pos(&self) -> BlockPos;
    fn face(&self) -> Face;
}

macro_rules! block_action {
    ($($event:ty),*) => {$(
        impl BlockActionExt for $event {
            fn pos(&self) -> BlockPos {
                BlockPos::from(self.pos)
            }

            fn face(&self) -> Face {
                Face::from(self.face)
            }
        }
    )*};
}

block_action!(BlockBreakStart, BlockBreakContinue, ItemUseOnBlock);
