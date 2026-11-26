use amml_abi::BuildBlock;

use crate::{BlockPos, Face, Hand};

/// Typed reads of a `BuildBlock`: a player is about to build at the clicked face of a block.
///
/// The event comes on the thread that builds: `AmmlThread::GAME` with the client in the context, or
/// `AmmlThread::SERVER` (the local-world server) with no client (`EventCtx::client()` gives `None`). A build that this
/// mod's own `GameMode::build_at` started has the origin `THIS_MOD`.
pub trait BuildBlockExt {
    /// The block that is clicked; the new block goes next to it, on the side of `face`.
    fn pos(&self) -> BlockPos;

    /// The clicked face.
    fn face(&self) -> Face;

    /// The hand that builds; a hand slot this crate does not know reads as `Hand::Main`.
    fn hand(&self) -> Hand;

    /// The game's isSimTick argument of the build.
    fn sim_tick(&self) -> bool;

    /// Whether the building player is the local player of the primary client (always false on the server thread).
    fn is_local_player(&self) -> bool;
}

impl BuildBlockExt for BuildBlock {
    fn pos(&self) -> BlockPos {
        BlockPos::from(self.pos)
    }

    fn face(&self) -> Face {
        Face::from(self.face)
    }

    fn hand(&self) -> Hand {
        Hand::from(self.hand)
    }

    fn sim_tick(&self) -> bool {
        self.sim_tick != 0
    }

    fn is_local_player(&self) -> bool {
        self.is_local_player != 0
    }
}
