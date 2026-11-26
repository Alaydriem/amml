use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::{Condvar, Mutex};
use std::time::{Duration, Instant};

use crate::ShutdownRun;

/// The once-per-process shutdown: whether it was queued for the game thread, whether it runs or ran, and a wait for
/// its end.
///
/// `run` lets one caller in for the life of the process: the live shutdown (`ModLoader::shutdown_live`) and the
/// shutdown after the game stopped (`ModLoader::shutdown`) both take it, so whichever comes first runs and the other
/// does nothing. The run ends when its `ShutdownRun` drops, also by a panic.
///
/// `is_closing` is true from `run` on, and also once the window thread passed a close on to the game before a queued
/// shutdown ran (`pass_on`): the game tears down then, whether or not a frame comes to run the shutdown. While it is
/// true the loader posts no size message. The render-thread hooks read `is_closing` to stop drawing; they release
/// their D3D objects on the render thread, never from here.
#[derive(Debug, Default)]
pub struct ShutdownLatch {
    state: AtomicU8,
    // The window thread passed the close on before the shutdown ran.
    passed_on: AtomicBool,
    // Held while `finish` stores `DONE`, so a waiter cannot miss the wake.
    lock: Mutex<()>,
    done: Condvar,
}

impl ShutdownLatch {
    const IDLE: u8 = 0;
    const QUEUED: u8 = 1;
    const RUNNING: u8 = 2;
    const DONE: u8 = 3;

    pub const fn new() -> Self {
        Self {
            state: AtomicU8::new(Self::IDLE),
            passed_on: AtomicBool::new(false),
            lock: Mutex::new(()),
            done: Condvar::new(),
        }
    }

    /// Asks the game thread to run the shutdown at its next frame. True when this call queued it; false when it was
    /// queued already, runs or ran.
    pub fn queue(&self) -> bool {
        self.state.compare_exchange(Self::IDLE, Self::QUEUED, Ordering::AcqRel, Ordering::Acquire).is_ok()
    }

    /// A shutdown waits for the game thread.
    pub fn is_queued(&self) -> bool {
        self.state.load(Ordering::Acquire) == Self::QUEUED
    }

    /// Lets the caller run the shutdown: `Some` once per process, for the first caller, queued or not, also after
    /// `pass_on`. The run is finished when the returned value drops.
    pub fn run(&self) -> Option<ShutdownRun<'_>> {
        let mut current = self.state.load(Ordering::Acquire);
        loop {
            if current != Self::IDLE && current != Self::QUEUED {
                return None;
            }
            match self.state.compare_exchange(current, Self::RUNNING, Ordering::AcqRel, Ordering::Acquire) {
                Ok(_) => return Some(ShutdownRun::new(self)),
                Err(found) => current = found,
            }
        }
    }

    /// The window thread passed the close on to the game before the shutdown ran: the loader counts as closing from
    /// here on. A later `run` is still allowed.
    pub fn pass_on(&self) {
        self.passed_on.store(true, Ordering::Release);
    }

    /// The shutdown runs or ran, or the game got the close before it ran.
    pub fn is_closing(&self) -> bool {
        self.state.load(Ordering::Acquire) >= Self::RUNNING || self.passed_on.load(Ordering::Acquire)
    }

    /// The shutdown ran; wakes every `wait_done`. `ShutdownRun` calls it when it drops.
    pub(crate) fn finish(&self) {
        let _held = self.lock.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        self.state.store(Self::DONE, Ordering::Release);
        self.done.notify_all();
    }

    /// Waits at most `limit` for the shutdown to finish. True when it finished.
    pub fn wait_done(&self, limit: Duration) -> bool {
        let deadline = Instant::now() + limit;
        let mut held = self.lock.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        while self.state.load(Ordering::Acquire) != Self::DONE {
            let left = deadline.saturating_duration_since(Instant::now());
            if left.is_zero() {
                return false;
            }
            held = match self.done.wait_timeout(held, left) {
                Ok((guard, _)) => guard,
                Err(poisoned) => poisoned.into_inner().0,
            };
        }
        true
    }
}
