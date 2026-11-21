use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, OnceLock};

use amml_abi::AmmlStatus;

use crate::{KeyDetours, KeyFeed, LoaderState, RealKeyPath, WindowProcDepth};

static FEED: OnceLock<Arc<ProcessorKeyFeed>> = OnceLock::new();

/// The key feed of the builds with `KeyboardEventProcessor::_feedKeyState` (shape `key_processor`): one
/// detour on that function.
///
/// An injected key calls the original with a null `this`, which the body never reads. The function is reached
/// only from the game window's key-message handler and its focus paths, so the key feed thread is the window
/// thread: injection is ready once the loader's window procedure ran on the game thread
/// (`window_on_game_thread`), or after a real key down on the game thread, whichever comes first; it is
/// `NOT_READY` before both, and `NOT_AVAILABLE` for good when a real key down runs off the game thread. The first
/// real key down logs whether it came through the window procedure. Until the hook is installed (`bind`), the detour passes every key to the
/// game untouched.
pub struct ProcessorKeyFeed {
    path: RealKeyPath,
    bound: AtomicBool,
    first_down_logged: AtomicBool,
}

impl ProcessorKeyFeed {
    const SHAPE: &'static str = "key_processor";

    /// Creates the feed once per process and gives it; `None` when it already exists.
    pub fn create(state: &'static LoaderState) -> Option<Arc<ProcessorKeyFeed>> {
        let feed = Arc::new(Self {
            path: RealKeyPath::new(state),
            bound: AtomicBool::new(false),
            first_down_logged: AtomicBool::new(false),
        });
        FEED.set(feed.clone()).ok().map(|()| feed)
    }

    /// Runs `body` on the feed; `fallback` before `create`.
    pub(crate) fn with<R>(fallback: R, body: impl FnOnce(&ProcessorKeyFeed) -> R) -> R {
        match FEED.get() {
            Some(feed) => body(feed),
            None => fallback,
        }
    }

    /// Marks the hook installed: from now on real keys run through the key path.
    pub fn bind(&self) {
        self.bound.store(true, Ordering::Release);
    }

    /// One real key state. Gives whether the game gets it.
    pub(crate) fn real(&self, key: u16, down: bool) -> bool {
        if !self.bound.load(Ordering::Acquire) {
            return true;
        }
        if down && !self.first_down_logged.swap(true, Ordering::AcqRel) {
            curia::info!("first real key down", {
                in_window_proc: WindowProcDepth::inside(),
                game_thread: self.path.on_game_thread()
            });
        }
        self.path.feed(key, down)
    }

    /// The loader's window procedure ran on the game thread: the key feed is the game thread.
    pub(crate) fn window_on_game_thread(&self) {
        if self.bound.load(Ordering::Acquire) {
            self.path.decide_from_window();
        }
    }
}

impl KeyFeed for ProcessorKeyFeed {
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

    fn send(&self, key: u16, down: bool) {
        if !KeyDetours::send_key_state(key, down) {
            curia::debug!("an injected key was dropped: the key hook is not installed", { key: key });
        }
    }

    // The game's reset loop that follows feeds ups of keys no longer down, which raise nothing.
    fn release_real(&self) {
        if self.bound.load(Ordering::Acquire) {
            self.path.release_held();
        }
    }
}
