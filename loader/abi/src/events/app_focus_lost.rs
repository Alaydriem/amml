use crate::{AmmlEvent, AmmlEventHeader};

/// The game is about to run its handler of an app focus loss (`MinecraftGame::onAppFocusLost`), which opens the
/// pause screen and resets the game's key state. A listener may cancel it: the game's handler then does not run, and
/// the window messages and the game's key reset, which do not pass this function, are not touched. A mod that
/// cancelled a loss can pass it on later with `AmmlClientTable::replay_app_focus_lost`.
///
/// Raised Before the game's handler, on the thread that runs it (`GAME` when that is the game thread, else `WINDOW`).
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct AppFocusLost {
    pub header: AmmlEventHeader,
}

impl AmmlEvent for AppFocusLost {
    const ID: &'static str = "client.app_focus_lost";
    const CANCELLABLE: bool = true;
}

const _: () = assert!(core::mem::size_of::<AppFocusLost>() == 16);
