use std::collections::{HashMap, HashSet};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::structs::{CallOutcome, DispatchReport, ListenError, ListenerEntry, ListenerId};

struct BusState<L> {
    offered: HashMap<String, u32>,
    listeners: HashMap<String, Vec<Arc<ListenerEntry<L>>>>,
    // Retired entries whose calls may still run; kept so a later remove or
    // remove_owner can wait for them.
    draining: Vec<Arc<ListenerEntry<L>>>,
    closed_owners: HashSet<usize>,
    next_id: u64,
}

/// Delivers events to listeners in band order, then registration order.
///
/// `dispatch` calls listeners with no lock held, so a listener may add or
/// remove other listeners. `remove` and `remove_owner` wait for calls in
/// progress on other threads; a listener that removes itself does not wait
/// for its own call. A retired listener stays reachable by `remove` and
/// `remove_owner` until its calls end. `remove_owner` closes the owner: `add`
/// for it fails with `OwnerClosed` until `reopen_owner`. Do not call
/// `remove_owner` from inside a listener of that owner (the caller's own call
/// is not waited for), and two listeners on two threads that remove each other
/// wait on each other forever.
pub struct EventBus<L: Clone + Send + Sync> {
    state: RwLock<BusState<L>>,
}

impl<L: Clone + Send + Sync> Default for EventBus<L> {
    fn default() -> Self {
        Self::new()
    }
}

impl<L: Clone + Send + Sync> EventBus<L> {
    pub fn new() -> Self {
        Self {
            state: RwLock::new(BusState {
                offered: HashMap::new(),
                listeners: HashMap::new(),
                draining: Vec::new(),
                closed_owners: HashSet::new(),
                next_id: 1,
            }),
        }
    }

    fn read(&self) -> RwLockReadGuard<'_, BusState<L>> {
        self.state.read().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn write(&self) -> RwLockWriteGuard<'_, BusState<L>> {
        self.state.write().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// True when `event` has at least one listener now.
    pub fn has_listeners(&self, event: &str) -> bool {
        self.read().listeners.get(event).is_some_and(|listeners| !listeners.is_empty())
    }

    /// Marks an event available up to `version`.
    pub fn offer(&self, event: &str, version: u32) {
        let mut state = self.write();
        let slot = state.offered.entry(event.to_owned()).or_insert(0);
        *slot = (*slot).max(version);
    }

    pub fn is_available(&self, event: &str, version: u32) -> bool {
        self.read().offered.get(event).is_some_and(|offered| *offered >= version)
    }

    pub fn add(
        &self,
        event: &str,
        version: u32,
        band: u8,
        owner: usize,
        listener: L,
    ) -> Result<ListenerId, ListenError> {
        let mut state = self.write();
        if state.closed_owners.contains(&owner) {
            return Err(ListenError::OwnerClosed);
        }
        match state.offered.get(event) {
            None => return Err(ListenError::UnknownEvent),
            Some(offered) if *offered < version => return Err(ListenError::VersionTooNew),
            Some(_) => {}
        }
        let id = ListenerId(state.next_id);
        state.next_id += 1;
        let entry = Arc::new(ListenerEntry::new(id, owner, band, listener));
        let list = state.listeners.entry(event.to_owned()).or_default();
        let at = list.partition_point(|existing| existing.band <= band);
        list.insert(at, entry);
        Ok(id)
    }

    /// Allows `add` for `owner` again after `remove_owner`.
    pub fn reopen_owner(&self, owner: usize) {
        self.write().closed_owners.remove(&owner);
    }

    /// Removes one listener and waits for its calls on other threads.
    /// Gives false for an unknown id, or for one removed earlier (after
    /// waiting for its calls that are still running).
    pub fn remove(&self, id: ListenerId) -> bool {
        let (entry, fresh) = {
            let mut state = self.write();
            state.draining.retain(|entry| !entry.is_idle());
            let mut found = None;
            for list in state.listeners.values_mut() {
                if let Some(at) = list.iter().position(|entry| entry.id == id) {
                    found = Some(list.remove(at));
                    break;
                }
            }
            match found {
                Some(entry) => {
                    // Refuse new calls while the bus lock is held, so no
                    // other thread can see this entry as idle and forget it
                    // before its calls are refused.
                    entry.close();
                    state.draining.push(entry.clone());
                    (Some(entry), true)
                }
                None => (state.draining.iter().find(|entry| entry.id == id).cloned(), false),
            }
        };
        match entry {
            Some(entry) => {
                entry.wait_idle();
                fresh
            }
            None => false,
        }
    }

    /// Closes `owner`, removes every listener of it and waits for their calls
    /// on other threads, including calls of listeners removed earlier. Gives
    /// how many listeners this call removed.
    pub fn remove_owner(&self, owner: usize) -> usize {
        let (fresh, earlier) = {
            let mut state = self.write();
            state.closed_owners.insert(owner);
            state.draining.retain(|entry| !entry.is_idle());
            let earlier: Vec<_> = state.draining.iter().filter(|entry| entry.owner == owner).cloned().collect();
            let mut taken = Vec::new();
            for list in state.listeners.values_mut() {
                let mut kept = Vec::with_capacity(list.len());
                for entry in list.drain(..) {
                    if entry.owner == owner {
                        taken.push(entry);
                    } else {
                        kept.push(entry);
                    }
                }
                *list = kept;
            }
            // Refuse new calls before the bus lock is released.
            for entry in &taken {
                entry.close();
            }
            state.draining.extend(taken.iter().cloned());
            (taken, earlier)
        };
        for entry in &earlier {
            entry.wait_idle();
        }
        for entry in &fresh {
            entry.wait_idle();
        }
        fresh.len()
    }

    /// Calls every listener of `event`. A `Cancelled` outcome stays in the
    /// report for the rest of the dispatch. After `Panicked`, `value` is the
    /// copy taken before that listener and the owner is listed.
    pub fn dispatch<E: Clone>(
        &self,
        event: &str,
        value: &mut E,
        mut call: impl FnMut(&L, &mut E) -> CallOutcome,
    ) -> DispatchReport {
        let snapshot: Vec<Arc<ListenerEntry<L>>> = self.read().listeners.get(event).cloned().unwrap_or_default();
        let mut report = DispatchReport::default();
        Self::call_all(&snapshot, value, &mut call, &mut report);
        report
    }

    /// `dispatch` with a pause: `between` runs once after the last listener of band `after_band` (also when that
    /// band is empty, or the event has no listeners) and before the next band. The value `between` gets is the one
    /// the earlier bands left.
    pub fn dispatch_split<E: Clone>(
        &self,
        event: &str,
        value: &mut E,
        after_band: u8,
        mut call: impl FnMut(&L, &mut E) -> CallOutcome,
        mut between: impl FnMut(&mut E),
    ) -> DispatchReport {
        let snapshot: Vec<Arc<ListenerEntry<L>>> = self.read().listeners.get(event).cloned().unwrap_or_default();
        let split = snapshot.partition_point(|entry| entry.band <= after_band);
        let mut report = DispatchReport::default();
        Self::call_all(&snapshot[..split], value, &mut call, &mut report);
        between(value);
        Self::call_all(&snapshot[split..], value, &mut call, &mut report);
        report
    }

    fn call_all<E: Clone>(
        entries: &[Arc<ListenerEntry<L>>],
        value: &mut E,
        call: &mut impl FnMut(&L, &mut E) -> CallOutcome,
        report: &mut DispatchReport,
    ) {
        for entry in entries {
            if !entry.begin_call() {
                continue;
            }
            let before = value.clone();
            let outcome = catch_unwind(AssertUnwindSafe(|| call(&entry.listener, value)));
            entry.end_call();
            match outcome {
                Ok(CallOutcome::Done) => {}
                Ok(CallOutcome::Cancelled) => report.cancelled = true,
                Ok(CallOutcome::Panicked) | Err(_) => {
                    *value = before;
                    report.panicked_owners.push(entry.owner);
                }
            }
        }
    }
}
