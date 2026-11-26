use crate::{AmmlEvent, AmmlEventHeader};

/// A player attacks an actor.
///
/// Raised on the game thread only, for every player; `is_local_player` is 1 for the local player of the
/// primary client. The context carries the primary client. A cancel stops the attack: the game gets "not handled". The target is not in version 1.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct PlayerAttack {
    pub header: AmmlEventHeader,
    pub is_local_player: u8,
    pub _reserved: [u8; 7],
}

impl AmmlEvent for PlayerAttack {
    const ID: &'static str = "player.attack";
    const CANCELLABLE: bool = true;
}

const _: () = assert!(core::mem::size_of::<PlayerAttack>() == 24);
