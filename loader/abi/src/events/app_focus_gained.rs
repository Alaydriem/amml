use crate::{AmmlEvent, AmmlEventHeader};

/// The game ran its handler of an app focus gain (`MinecraftGame::onAppFocusGained`). A notification: it cannot be
/// cancelled. A focus gain ends every held focus loss: the game did not see the loss, so a later replay is not
/// needed.
///
/// Raised After the game's handler, on the thread that runs it (`GAME` when that is the game thread, else `WINDOW`).
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct AppFocusGained {
    pub header: AmmlEventHeader,
}

impl AmmlEvent for AppFocusGained {
    const ID: &'static str = "client.app_focus_gained";
}

const _: () = assert!(core::mem::size_of::<AppFocusGained>() == 16);
