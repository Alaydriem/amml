use crate::{AmmlEvent, AmmlEventHeader};

/// The player presses the use (build) action: the game is about to run `ClientInputCallbacks::handleBuildAction` for
/// the press. Held repeats of the use do not come through here.
///
/// Raised Before the game's handler, on the game thread, with the primary client in the context. A cancel keeps the
/// handler from running: the game gets the "handled" value (`true`) and builds or uses nothing for this press.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct UsePressed {
    pub header: AmmlEventHeader,
}

impl AmmlEvent for UsePressed {
    const ID: &'static str = "player.use_pressed";
    const CANCELLABLE: bool = true;
}

const _: () = assert!(core::mem::size_of::<UsePressed>() == 16);
