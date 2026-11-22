use crate::{AmmlBlockPos, AmmlEvent, AmmlEventHeader, AmmlFace, AmmlHand};

/// A player builds a block: the game is about to run the base body of `GameMode::buildBlock` (`pos` is the block that
/// is clicked, `face` the clicked face, `hand` the hand slot, `sim_tick` the game's isSimTick argument).
///
/// Raised on the thread that builds: `AmmlThread::GAME` with the primary client in the context, or
/// `AmmlThread::SERVER` (the local-world server) with no client. `is_local_player` is 1 when the game mode's player
/// is the local player of the primary client (always 0 on the server thread). A build that a mod's own `build_at`
/// started has the origin `THIS_MOD` for the listeners of that mod and `OTHER_MOD` for the others. One event per
/// call. A cancel stops the build: the game's call returns `false` and no block is placed.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct BuildBlock {
    pub header: AmmlEventHeader,
    pub pos: AmmlBlockPos,
    pub face: AmmlFace,
    pub hand: AmmlHand,
    pub sim_tick: u8,
    pub is_local_player: u8,
}

impl AmmlEvent for BuildBlock {
    const ID: &'static str = "player.build_block";
    const CANCELLABLE: bool = true;
}

const _: () = assert!(core::mem::size_of::<BuildBlock>() == 32);
