use std::sync::{Mutex, MutexGuard};

/// The item stacks a loader call lent, each with the scope that was running when it was lent.
///
/// A scope is a number the caller makes from what runs now (the game frame and the UI dispatch). A lent stack
/// is accepted only while the same scope runs; lending in another scope drops every older entry, so the ledger
/// never holds more than the stacks of the newest scope. The stack the player holds is not in the ledger: the
/// caller passes it as `held`.
#[derive(Debug, Default)]
pub struct StackLedger {
    lent: Mutex<Vec<(u64, usize)>>,
}

impl StackLedger {
    pub fn new() -> Self {
        Self { lent: Mutex::new(Vec::new()) }
    }

    /// Records that `stack` was lent while `scope` ran. Entries of other scopes are dropped.
    pub fn lend(&self, scope: u64, stack: usize) {
        let mut lent = self.lock();
        lent.retain(|(known, _)| *known == scope);
        if !lent.contains(&(scope, stack)) {
            lent.push((scope, stack));
        }
    }

    /// Forgets every stack lent while `scope` ran: a stack lent before is refused from now on.
    pub fn forget(&self, scope: u64) {
        self.lock().retain(|(known, _)| *known != scope);
    }

    /// Whether a call made in `scope` may read `stack`: it is not null and it is one of `held` or was lent in
    /// `scope`.
    pub fn accepts(&self, scope: u64, stack: usize, held: &[usize]) -> bool {
        stack != 0 && (held.contains(&stack) || self.lock().contains(&(scope, stack)))
    }

    /// The number of stacks the ledger holds.
    pub fn len(&self) -> usize {
        self.lock().len()
    }

    /// Whether the ledger holds no stack.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn lock(&self) -> MutexGuard<'_, Vec<(u64, usize)>> {
        self.lent.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}
