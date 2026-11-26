use std::ffi::c_void;
use std::sync::atomic::{AtomicBool, AtomicI32, AtomicUsize, Ordering};
use std::sync::{Arc, OnceLock};

use amml_abi::AmmlStatus;

use crate::{KeyDetours, KeyFeed, LoaderState, RealKeyPath};

static FEED: OnceLock<Arc<HidKeyFeed>> = OnceLock::new();

/// The key feed of the builds with `HIDControllerGameCoreDesktop` (shape `hid`): detours on its
/// `onKeyDown`, `onKeyUp` and `update`, and on the `HIDController` key reset when that entry resolved.
/// The key reset writes the game's key table without `onKeyUp`, so its detour releases the loader's held
/// keys first (`release_real`).
///
/// An injected key is sent to the keyboard controller the loader last saw (from `update`, or the `this` of
/// the last real key), with the origin of the last real key down (1, hardware, before the first one).
/// Injection is `NOT_READY` until a controller is known. Until every hook is installed (`bind`), the
/// detours pass every key to the game untouched, so a partly refused hook set cannot hold a key back.
pub struct HidKeyFeed {
    path: RealKeyPath,
    bound: AtomicBool,
    controller: AtomicUsize,
    origin: AtomicI32,
}

impl HidKeyFeed {
    const SHAPE: &'static str = "hid";
    // The origin the game passes for a hardware key.
    const DEFAULT_ORIGIN: i32 = 1;

    /// Creates the feed once per process and gives it; `None` when it already exists.
    pub fn create(state: &'static LoaderState) -> Option<Arc<HidKeyFeed>> {
        let feed = Arc::new(Self {
            path: RealKeyPath::new(state),
            bound: AtomicBool::new(false),
            controller: AtomicUsize::new(0),
            origin: AtomicI32::new(Self::DEFAULT_ORIGIN),
        });
        FEED.set(feed.clone()).ok().map(|()| feed)
    }

    /// Runs `body` on the feed; `fallback` before `create`.
    pub(crate) fn with<R>(fallback: R, body: impl FnOnce(&HidKeyFeed) -> R) -> R {
        match FEED.get() {
            Some(feed) => body(feed),
            None => fallback,
        }
    }

    /// Marks every hook of the feed installed: from now on real keys run through the key path.
    pub fn bind(&self) {
        self.bound.store(true, Ordering::Release);
    }

    /// One real key down on `controller`. Gives whether the game gets it.
    pub(crate) fn real_down(&self, controller: *mut c_void, key: u16, origin: i32) -> bool {
        if !self.bound.load(Ordering::Acquire) {
            return true;
        }
        self.note_controller(controller);
        self.origin.store(origin, Ordering::Release);
        self.path.feed(key, true)
    }

    /// One real key up on `controller`. Gives whether the game gets it.
    pub(crate) fn real_up(&self, controller: *mut c_void, key: u16) -> bool {
        if !self.bound.load(Ordering::Acquire) {
            return true;
        }
        self.note_controller(controller);
        self.path.feed(key, false)
    }

    /// Keeps `controller` as the target of injected keys; null changes nothing.
    pub(crate) fn note_controller(&self, controller: *mut c_void) {
        let address = controller.expose_provenance();
        if address != 0 {
            self.controller.store(address, Ordering::Release);
        }
    }
}

impl KeyFeed for HidKeyFeed {
    fn shape(&self) -> &'static str {
        Self::SHAPE
    }

    fn ready(&self) -> Result<(), AmmlStatus> {
        self.path.thread_allows()?;
        if self.controller.load(Ordering::Acquire) == 0 {
            return Err(AmmlStatus::NOT_READY);
        }
        Ok(())
    }

    fn send(&self, key: u16, down: bool) {
        let controller = self.controller.load(Ordering::Acquire);
        if controller == 0 {
            return;
        }
        let controller = std::ptr::with_exposed_provenance_mut::<c_void>(controller);
        // SAFETY: `controller` is the keyboard controller the game passed to its own `update` or key function.
        let sent = unsafe {
            if down {
                KeyDetours::send_key_down(controller, key, self.origin.load(Ordering::Acquire))
            } else {
                KeyDetours::send_key_up(controller, key)
            }
        };
        if !sent {
            curia::debug!("an injected key was dropped: the key hook is not installed", { key: key });
        }
    }

    fn release_real(&self) {
        if self.bound.load(Ordering::Acquire) {
            self.path.release_held();
        }
    }
}
