use std::ffi::c_void;
use std::sync::{Arc, OnceLock};

use amml_abi::{AmmlMouseButton, AmmlStatus};

use crate::{LoaderState, MouseDetours, MouseFeed, RealMousePath};

static FEED: OnceLock<Arc<EightArgMouseFeed>> = OnceLock::new();

/// The mouse feed of the builds with the eight-argument `MouseDevice::feed` (70.x, shape `eight_arg_feed`):
/// one detour on that function. The eighth argument, the controller id, passes through unchanged on the
/// real path.
///
/// Injected input is fed to `MouseDevice.instance` at the pointer position of the last real feed, with
/// controller id 0, as every desktop caller of the game passes. Injection is `NOT_READY` until a real feed
/// has shown which thread feeds the mouse, and `NOT_AVAILABLE` for good when that is not the game thread.
pub struct EightArgMouseFeed {
    path: RealMousePath,
}

impl EightArgMouseFeed {
    /// The entry of the eight-argument feed.
    pub const FEED_ENTRY: &'static str = "MouseDevice.feedWithControllerId";
    /// The controller id the game's 14 direct callers pass.
    pub const DESKTOP_CONTROLLER: i32 = 0;
    const SHAPE: &'static str = "eight_arg_feed";

    /// Creates the feed once per process for the device at `instance` and gives it; `None` when it
    /// already exists.
    pub fn create(state: &'static LoaderState, instance: usize) -> Option<Arc<EightArgMouseFeed>> {
        let feed = Arc::new(Self { path: RealMousePath::new(state, instance) });
        FEED.set(feed.clone()).ok().map(|()| feed)
    }

    /// Runs `body` on the feed; `fallback` before `create`.
    pub(crate) fn with<R>(fallback: R, body: impl FnOnce(&EightArgMouseFeed) -> R) -> R {
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
        // SAFETY: `device` is the game's static `MouseDevice`; `send_button` and `send_wheel` run on the game
        // thread, which `ready` has checked is the thread of the real feed.
        let sent = unsafe { MouseDetours::send_eight_arg(device, action, data, x, y, Self::DESKTOP_CONTROLLER) };
        if !sent {
            curia::debug!("an injected mouse action was dropped: the mouse hook is not installed", { action: action });
        }
    }
}

impl MouseFeed for EightArgMouseFeed {
    fn shape(&self) -> &'static str {
        Self::SHAPE
    }

    fn ready(&self) -> Result<(), AmmlStatus> {
        self.path.thread_allows()?;
        if !self.path.decided() {
            return Err(AmmlStatus::NOT_READY);
        }
        Ok(())
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
