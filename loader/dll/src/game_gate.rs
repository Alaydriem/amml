use std::sync::Mutex;
use std::sync::atomic::{AtomicU8, AtomicU64, Ordering};
use std::thread::ThreadId;

use amml_abi::AmmlStatus;
use amml_runtime::WorldEpoch;

/// Whether game code may be called yet, the epoch of the current world and the count of worlds joined.
///
/// The gate opens at the first `FrameUpdate`; before it (in `load` and `enable`, which can run
/// before the game's static initializers) every game read and action gives `NOT_READY`. It closes
/// again at shutdown, when the game no longer runs, and stays closed: a later `FrameUpdate` does
/// not open it again.
#[derive(Debug, Default)]
pub struct GameGate {
    state: AtomicU8,
    epoch: AtomicU64,
    joins: AtomicU64,
    game_thread: Mutex<Option<ThreadId>>,
}

impl GameGate {
    const WAITING: u8 = 0;
    const OPEN: u8 = 1;
    const SHUT: u8 = 2;

    pub const fn new() -> Self {
        Self {
            state: AtomicU8::new(Self::WAITING),
            epoch: AtomicU64::new(0),
            joins: AtomicU64::new(0),
            game_thread: Mutex::new(None),
        }
    }

    pub fn is_open(&self) -> bool {
        self.state.load(Ordering::Acquire) == Self::OPEN
    }

    /// `Ok` once the gate is open, else `NOT_READY`. Every game read and action starts with it.
    pub fn require_open(&self) -> Result<(), AmmlStatus> {
        if self.is_open() { Ok(()) } else { Err(AmmlStatus::NOT_READY) }
    }

    /// Opens the gate, unless it was closed for good. The first `FrameUpdate` calls it.
    pub fn open(&self) {
        let _ = self.state.compare_exchange(Self::WAITING, Self::OPEN, Ordering::AcqRel, Ordering::Acquire);
    }

    /// Records the calling thread as the game thread. Every frame calls it.
    pub fn note_game_thread(&self) {
        *self.game_thread.lock().unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(std::thread::current().id());
    }

    /// Whether the calling thread is the thread of the last frame; false before the first frame.
    pub fn on_game_thread(&self) -> bool {
        *self.game_thread.lock().unwrap_or_else(|poisoned| poisoned.into_inner()) == Some(std::thread::current().id())
    }

    /// Closes the gate for good: the game is shutting down.
    pub fn close(&self) {
        self.state.store(Self::SHUT, Ordering::Release);
    }

    /// The epoch of the current world; 0 when no world is joined.
    pub fn epoch(&self) -> WorldEpoch {
        WorldEpoch(self.epoch.load(Ordering::Acquire))
    }

    /// Starts a new world and gives its epoch. Every world gets a new epoch, also after a leave.
    pub fn next_epoch(&self) -> WorldEpoch {
        let epoch = self.joins.fetch_add(1, Ordering::AcqRel) + 1;
        self.epoch.store(epoch, Ordering::Release);
        WorldEpoch(epoch)
    }

    /// Ends the current world: the epoch is 0 until the next join.
    pub fn end_world(&self) {
        self.epoch.store(0, Ordering::Release);
    }
}
