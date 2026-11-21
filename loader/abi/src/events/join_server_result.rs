use crate::{AmmlEvent, AmmlEventHeader, AmmlJoinOutcome};

/// The game answered the server join `join` (the id `ClientJoinServerFn` gave) with `outcome`.
///
/// Raised on thread `GAME` at the next `FrameUpdate` after the answer, to the mod that asked only. It comes once for
/// every join: when the game gives no answer before the world is joined, it comes then with `OTHER`. A mod disabled
/// before it is raised does not get it.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct JoinServerResult {
    pub header: AmmlEventHeader,
    pub join: u64,
    pub outcome: AmmlJoinOutcome,
    pub _reserved: u32,
}

impl AmmlEvent for JoinServerResult {
    const ID: &'static str = "client.join_server_result";
}

const _: () = assert!(core::mem::size_of::<JoinServerResult>() == 32);
