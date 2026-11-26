use crate::{AmmlEvent, AmmlEventHeader, AmmlMoveIntent};

/// The movement the player input gives this frame.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct PlayerMoveInput {
    pub header: AmmlEventHeader,
    pub intent: AmmlMoveIntent,
}

impl AmmlEvent for PlayerMoveInput {
    const ID: &'static str = "input.player_move";
}

const _: () = assert!(core::mem::size_of::<PlayerMoveInput>() == 28);
