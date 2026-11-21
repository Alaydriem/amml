use amml_abi::CameraInstructionReceived;

use crate::InstructionKind;

/// Typed reads of a `CameraInstructionReceived`: the client is about to handle a camera instruction, Before the game's
/// handler, on the game thread.
///
/// `event.header().origin` says who sent it: `GAME` for the server, `THIS_MOD` for an instruction of this mod's own
/// `CameraControl::set_free` or `release_free`, `OTHER_MOD` for another mod's. A mod that holds the free camera and sees
/// a server instruction (origin `GAME`) knows the server moved the camera.
pub trait CameraInstructionReceivedExt {
    /// What the instruction asks.
    fn kind(&self) -> InstructionKind;
}

impl CameraInstructionReceivedExt for CameraInstructionReceived {
    fn kind(&self) -> InstructionKind {
        InstructionKind::from_abi(self.kind)
    }
}
