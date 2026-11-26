use crate::{AmmlEvent, AmmlEventHeader, AmmlRotation, AmmlVec3};

/// The game handled a `MovePlayerPacket` of the local player: raised After the game's handler, on the game thread.
/// The packets of other actors raise nothing. A packet of the local player that comes before the world join (the
/// server's Respawn right after `StartGame`) is raised right after `WorldJoined`. The context carries the primary client,
/// so a listener reaches the local player through it.
///
/// `pos` is the packet's position, `rotation` its pitch and yaw in degrees with the signs of `AmmlRotation` (the angle
/// rule of `actor.read`), `head_yaw` the head yaw in degrees and `mode` the packet's mode byte (the reset position
/// field).
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct PlayerMoved {
    pub header: AmmlEventHeader,
    pub pos: AmmlVec3,
    pub rotation: AmmlRotation,
    pub head_yaw: f32,
    pub mode: u8,
    pub _reserved: [u8; 7],
}

impl AmmlEvent for PlayerMoved {
    const ID: &'static str = "player.moved";
}

const _: () = assert!(core::mem::size_of::<PlayerMoved>() == 48);
