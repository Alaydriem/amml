use std::sync::{Mutex, MutexGuard};

use crate::OwnedItemStack;

// The stack the running pick replaced in its hotbar slot, kept for the undo. One request runs at a time for the whole
// loader, so one slot of memory is enough.
static REPLACED: Mutex<Option<(u8, OwnedItemStack)>> = Mutex::new(None);

/// The stack a creative pick replaced, as a copy the game made, until the pick ends: given back when the server refuses
/// the pick or does not answer, dropped when it accepts. Process-global because the game acts of a request run in a port
/// that is built for one tick.
pub struct PickUndo;

impl PickUndo {
    // The memory holds an option of an owned stack, so a lock poisoned by a panic is recovered.
    fn lock() -> MutexGuard<'static, Option<(u8, OwnedItemStack)>> {
        REPLACED.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    /// Keeps `stack` as the one `slot` held before the pick (none for a slot that was empty); a copy kept before is
    /// destroyed.
    pub fn keep(slot: u8, stack: Option<OwnedItemStack>) {
        *Self::lock() = stack.map(|stack| (slot, stack));
    }

    /// The stack kept for `slot`; `None` when none is kept for it.
    pub fn take(slot: u8) -> Option<OwnedItemStack> {
        let mut kept = Self::lock();
        match kept.take() {
            Some((kept_slot, stack)) if kept_slot == slot => Some(stack),
            other => {
                *kept = other;
                None
            }
        }
    }

    /// Destroys the kept stack: the pick was accepted.
    pub fn forget() {
        *Self::lock() = None;
    }
}
