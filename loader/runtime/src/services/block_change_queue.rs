use std::collections::VecDeque;
use std::sync::{Mutex, MutexGuard};

use crate::BlockChange;

/// The block changes the game made off the game thread, until the next frame raises them.
///
/// The queue holds at most `CAPACITY` changes; one more is dropped and counted. Each change carries the world epoch it
/// was made in: a change of another epoch than the one asked for is never given out, and `clear` (a world leave)
/// forgets every change. Taking the changes gives the number dropped since the last take, so the caller logs it once
/// for a frame.
pub struct BlockChangeQueue {
    inner: Mutex<Inner>,
}

struct Inner {
    changes: VecDeque<(u64, BlockChange)>,
    dropped: u64,
}

impl BlockChangeQueue {
    /// The most changes the queue holds.
    pub const CAPACITY: usize = 4096;

    pub fn new() -> Self {
        Self { inner: Mutex::new(Inner { changes: VecDeque::new(), dropped: 0 }) }
    }

    /// Keeps `change`, made in world `epoch`. False, and the drop counted, when the queue is full.
    pub fn push(&self, epoch: u64, change: BlockChange) -> bool {
        let mut inner = self.lock();
        if inner.changes.len() >= Self::CAPACITY {
            inner.dropped = inner.dropped.saturating_add(1);
            return false;
        }
        inner.changes.push_back((epoch, change));
        true
    }

    /// Gives every kept change made in world `epoch` to `raise`, in the order they were pushed, and empties the queue
    /// (a change of another epoch is forgotten). Returns the number of changes dropped for lack of room since the last
    /// call. `raise` runs with no lock held.
    pub fn take(&self, epoch: u64, mut raise: impl FnMut(&BlockChange)) -> u64 {
        let (changes, dropped) = {
            let mut inner = self.lock();
            (std::mem::take(&mut inner.changes), std::mem::take(&mut inner.dropped))
        };
        for (made_in, change) in &changes {
            if *made_in == epoch {
                raise(change);
            }
        }
        dropped
    }

    /// Forgets every kept change and the count of drops. A world leave calls it.
    pub fn clear(&self) {
        let mut inner = self.lock();
        inner.changes.clear();
        inner.dropped = 0;
    }

    fn lock(&self) -> MutexGuard<'_, Inner> {
        self.inner.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

impl Default for BlockChangeQueue {
    fn default() -> Self {
        Self::new()
    }
}
