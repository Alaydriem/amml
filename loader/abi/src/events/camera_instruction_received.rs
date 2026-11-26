use crate::{AmmlEvent, AmmlEventHeader, AmmlInstructionKind};

/// The client's network handler is about to handle a camera instruction (`ClientNetworkHandler::handle` of a
/// `CameraInstructionPacket`): raised Before the game's handler, on the game thread.
///
/// `kind` is what the instruction asks: `SET`, `CLEAR`, or `OTHER` (both parts or neither). The header `origin` is
/// `GAME` for an instruction of the server; for an instruction the loader hands to the handler for a mod's
/// `set_free` or `release_free` it is `THIS_MOD` for the listeners of that mod and `OTHER_MOD` for the others
/// (`injected` is 1). Not cancellable: the game always handles the instruction. No field is writable: the loader writes
/// `kind` and `_reserved` back after each listener. `_reserved` stays reserved; a later version appends its fields
/// after it, so its size is 24 or more.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct CameraInstructionReceived {
    pub header: AmmlEventHeader,
    pub kind: AmmlInstructionKind,
    pub _reserved: [u8; 7],
}

impl AmmlEvent for CameraInstructionReceived {
    const ID: &'static str = "camera.instruction_received";
}

const _: () = assert!(core::mem::size_of::<CameraInstructionReceived>() == 24);
