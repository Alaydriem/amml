use super::Locked;
use crate::BridgeError;
use amml_gdk::{CancelFlag, GdkError};
use amml_launcher_core::LauncherError;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, MutexGuard, TryLockError};
use std::time::Duration;

// Holds the one manager. It is built on first use. A job holds it for as long as the job runs. A reader that must not
// wait for a job uses `run_unless_busy`; a job that must wait for a reader uses `run_waiting`.
pub struct ManagerSlot<M> {
    slot: Mutex<Option<M>>,
    // set by `reset`; the next caller that holds the lock drops the manager instead of using or keeping it
    stale: AtomicBool,
}

impl<M> ManagerSlot<M> {
    // how long a caller sleeps between two tries of the lock
    pub const POLL: Duration = Duration::from_millis(20);

    pub fn new() -> ManagerSlot<M> {
        ManagerSlot { slot: Mutex::new(None), stale: AtomicBool::new(false) }
    }

    // Runs `f` on the manager, building it first when there is none. Waits while another caller uses it.
    pub fn run<R, E: From<LauncherError>>(
        &self,
        build: impl FnOnce() -> Result<M, LauncherError>,
        f: impl FnOnce(&mut M) -> Result<R, E>,
    ) -> Result<R, E> {
        self.use_slot(Locked::of(&self.slot), build, f)
    }

    // Like `run`, but a reader does not wait for a job: while the lock is held it asks `busy` every `POLL` and
    // answers `Busy` as soon as it says yes.
    pub fn run_unless_busy<R>(
        &self,
        busy: impl Fn() -> bool,
        build: impl FnOnce() -> Result<M, LauncherError>,
        f: impl FnOnce(&mut M) -> Result<R, BridgeError>,
    ) -> Result<R, BridgeError> {
        loop {
            match self.slot.try_lock() {
                Ok(slot) => return self.use_slot(slot, build, f),
                Err(TryLockError::Poisoned(poisoned)) => return self.use_slot(poisoned.into_inner(), build, f),
                Err(TryLockError::WouldBlock) => {
                    if busy() {
                        return Err(BridgeError::Busy);
                    }
                    std::thread::sleep(Self::POLL);
                }
            }
        }
    }

    // Like `run`, for a job: it tries the lock every `POLL`, ends as cancelled when `cancel` is set, and tells `wait`
    // with `true` when it first has to wait and with `false` when it has the lock after waiting.
    pub fn run_waiting<R, E: From<LauncherError> + From<GdkError>>(
        &self,
        cancel: &CancelFlag,
        wait: impl Fn(bool),
        build: impl FnOnce() -> Result<M, LauncherError>,
        f: impl FnOnce(&mut M) -> Result<R, E>,
    ) -> Result<R, E> {
        let mut waited = false;
        let slot = loop {
            match self.slot.try_lock() {
                Ok(slot) => break slot,
                Err(TryLockError::Poisoned(poisoned)) => break poisoned.into_inner(),
                Err(TryLockError::WouldBlock) => {
                    if cancel.is_cancelled() {
                        return Err(GdkError::Cancelled.into());
                    }
                    if !waited {
                        waited = true;
                        wait(true);
                    }
                    std::thread::sleep(Self::POLL);
                }
            }
        };
        if waited {
            wait(false);
        }
        self.use_slot(slot, build, f)
    }

    // Marks the manager as out of date, so the next use builds it from the settings on disk. It does not wait for a
    // caller that holds the manager: that caller drops the manager when it is done.
    pub fn reset(&self) {
        self.stale.store(true, Ordering::SeqCst);
    }

    fn use_slot<R, E: From<LauncherError>>(
        &self,
        mut slot: MutexGuard<'_, Option<M>>,
        build: impl FnOnce() -> Result<M, LauncherError>,
        f: impl FnOnce(&mut M) -> Result<R, E>,
    ) -> Result<R, E> {
        let kept = slot.take();
        let mut manager = match kept {
            Some(manager) if !self.stale.swap(false, Ordering::SeqCst) => manager,
            _ => {
                self.stale.store(false, Ordering::SeqCst);
                build()?
            }
        };
        let result = f(&mut manager);
        // a reset during `f` means this manager was built from settings that have changed
        if !self.stale.swap(false, Ordering::SeqCst) {
            *slot = Some(manager);
        }
        result
    }
}

impl<M> Default for ManagerSlot<M> {
    fn default() -> Self {
        Self::new()
    }
}
