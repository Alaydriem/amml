use std::collections::HashMap;

use crate::structs::{ChainChange, ChainError, HookEntry, OriginalValue, OriginalWrite};

/// Per target address, the hooks in call order.
///
/// Higher priority runs first; equal priority keeps registration order. Holds at most `max_hooks`
/// hooks over all targets; `add` refuses the next one, and `remove` frees its place.
pub struct HookChain {
    max_hooks: usize,
    chains: HashMap<u64, Vec<HookEntry>>,
    targets: HashMap<u64, u64>,
}

impl HookChain {
    pub fn new(max_hooks: usize) -> Self {
        Self { max_hooks, chains: HashMap::new(), targets: HashMap::new() }
    }

    /// Number of hooks over all targets.
    pub fn len(&self) -> usize {
        self.targets.len()
    }

    pub fn is_empty(&self) -> bool {
        self.targets.is_empty()
    }

    /// The target a hook was added to.
    pub fn target_of(&self, handle: u64) -> Option<u64> {
        self.targets.get(&handle).copied()
    }

    pub fn add(
        &mut self,
        target: u64,
        handle: u64,
        priority: i32,
        detour: u64,
        original_slot: u64,
    ) -> Result<ChainChange, ChainError> {
        if self.targets.contains_key(&handle) {
            return Err(ChainError::DuplicateHandle(handle));
        }
        if self.targets.len() >= self.max_hooks {
            return Err(ChainError::Full(self.max_hooks));
        }
        if self.chains.values().flatten().any(|hook| hook.original_slot == original_slot) {
            return Err(ChainError::DuplicateSlot(original_slot));
        }
        if self.chains.get(&target).is_some_and(|chain| chain.iter().any(|hook| hook.detour == detour)) {
            return Err(ChainError::DuplicateDetour(detour));
        }
        let chain = self.chains.entry(target).or_default();
        // after every hook with the same or a higher priority
        let at = chain.partition_point(|hook| hook.priority >= priority);
        chain.insert(at, HookEntry { handle, priority, detour, original_slot });
        self.targets.insert(handle, target);
        Ok(Self::change(target, chain))
    }

    pub fn remove(&mut self, handle: u64) -> Result<ChainChange, ChainError> {
        let target = self.targets.remove(&handle).ok_or(ChainError::UnknownHandle(handle))?;
        let Some(chain) = self.chains.get_mut(&target) else {
            return Err(ChainError::UnknownHandle(handle));
        };
        chain.retain(|hook| hook.handle != handle);
        let change = Self::change(target, chain);
        if chain.is_empty() {
            self.chains.remove(&target);
        }
        Ok(change)
    }

    fn change(target: u64, chain: &[HookEntry]) -> ChainChange {
        let originals = chain
            .iter()
            .enumerate()
            .rev()
            .map(|(index, hook)| OriginalWrite {
                slot: hook.original_slot,
                value: chain.get(index + 1).map_or(OriginalValue::Trampoline, |next| OriginalValue::Hook(next.detour)),
            })
            .collect();
        ChainChange { target, first: chain.first().map(|hook| hook.detour), originals }
    }
}
