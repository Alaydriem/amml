use crate::{AmmlEvent, AmmlEventHeader};

/// The game drew a screen view (`ui.screen_render`, After the draw). `screen` is the id of the view's screen
/// controller and `view` the id of the view; both are nonzero ids that are never used again. `tick` is the
/// number of `ClientTick` events raised so far. The `ui` table answers questions about `view` only while this
/// event's dispatch runs.
///
/// Raised on the thread that draws the view: `GAME` when it is the game thread, else `RENDER`.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ScreenRender {
    pub header: AmmlEventHeader,
    pub screen: u64,
    pub view: u64,
    pub tick: u64,
}

impl AmmlEvent for ScreenRender {
    const ID: &'static str = "ui.screen_render";
}

const _: () = assert!(core::mem::size_of::<ScreenRender>() == 40);
