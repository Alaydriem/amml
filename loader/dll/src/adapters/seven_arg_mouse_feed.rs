use std::ffi::c_void;
use std::sync::{Arc, OnceLock};

use amml_abi::{AmmlMouseButton, AmmlStatus};

use crate::{LoaderState, MouseDetours, MouseFeed, RealMousePath};

static FEED: OnceLock<Arc<SevenArgMouseFeed>> = OnceLock::new();

/// The mouse feed of the builds with the seven-argument `MouseDevice::feed` (26.6x, shape
/// `seven_arg_feed`): one detour on that function.
///
/// Injected input is fed to `MouseDevice.instance` at the pointer position of the last real feed.
/// Injection is `NOT_READY` until the first frame, then ready with no real feed needed (the position is 0, 0 until
/// one comes), and `NOT_AVAILABLE` for good when a real feed shows that the mouse is not fed on the game thread.
pub struct SevenArgMouseFeed {
    path: RealMousePath,
}

impl SevenArgMouseFeed {
    /// The entry of the seven-argument feed.
    pub const FEED_ENTRY: &'static str = "MouseDevice.feed";
    const SHAPE: &'static str = "seven_arg_feed";

    /// Creates the feed once per process for the device at `instance` and gives it; `None` when it
    /// already exists.
    pub fn create(state: &'static LoaderState, instance: usize) -> Option<Arc<SevenArgMouseFeed>> {
        let feed = Arc::new(Self { path: RealMousePath::new(state, instance) });
        FEED.set(feed.clone()).ok().map(|()| feed)
    }

    /// Runs `body` on the feed; `fallback` before `create`.
    pub(crate) fn with<R>(fallback: R, body: impl FnOnce(&SevenArgMouseFeed) -> R) -> R {
        match FEED.get() {
            Some(feed) => body(feed),
            None => fallback,
        }
    }

    /// One real feed; see `RealMousePath::feed`.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn real(
        &self,
        this: usize,
        action: u8,
        data: i8,
        x: i16,
        y: i16,
        dx: i16,
        dy: i16,
    ) -> Option<(i16, i16)> {
        self.path.feed(this, action, data, x, y, dx, dy)
    }

    // Feeds `action` with `data` to the device at the last real position.
    fn send(&self, action: u8, data: i8) {
        let (x, y) = self.path.last_position();
        let device = std::ptr::with_exposed_provenance_mut::<c_void>(self.path.instance());
        // SAFETY: `device` is the game's static `MouseDevice`; `send_button` and `send_wheel` run in the frame
        // flush on the game thread; `ready` refuses injection once a real feed has run off the game thread
        // (`RealMousePath::ready`).
        if !unsafe { MouseDetours::send_seven_arg(device, action, data, x, y) } {
            curia::debug!("an injected mouse action was dropped: the mouse hook is not installed", { action: action });
        }
    }
}

impl MouseFeed for SevenArgMouseFeed {
    fn shape(&self) -> &'static str {
        Self::SHAPE
    }

    fn ready(&self) -> Result<(), AmmlStatus> {
        self.path.ready()
    }

    fn send_button(&self, button: AmmlMouseButton, down: bool) {
        if let Some(action) = RealMousePath::action_of(button) {
            self.send(action, i8::from(down));
        }
    }

    fn send_wheel(&self, notches: f32) {
        let notch = if notches < 0.0 { -RealMousePath::WHEEL_NOTCH } else { RealMousePath::WHEEL_NOTCH };
        for _ in 0..RealMousePath::whole_notches(notches) {
            self.send(RealMousePath::WHEEL, notch);
        }
    }
}
