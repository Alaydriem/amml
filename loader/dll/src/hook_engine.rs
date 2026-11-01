use std::collections::HashMap;
use std::ffi::c_void;
use std::sync::atomic::{AtomicPtr, Ordering};

use amml_runtime::{ChainChange, ChainError, HookChain, OriginalValue};

use crate::HookError;
use crate::target_hook::TargetHook;

/// Owns every detour: one per target address, however many mods hook it.
///
/// A target is detoured once, to a fixed thunk. The thunk jumps to the first hook of the chain, and
/// each hook's `original` slot holds the next hook's detour or the trampoline. A change to the chain
/// writes slots and then moves the thunk, all with single atomic stores; the detour itself is never
/// patched again, and never dropped while the engine lives, so the trampoline stays valid for every
/// thread that is still inside a hook that was removed. An empty chain points the thunk at the
/// trampoline.
///
/// Slot writes are `Release`; mods read their `original` slot with `Acquire`.
///
/// Installing the first hook on a target patches its prologue, which is not atomic against a thread
/// running that code; do it in the load/enable phase, before the game runs the target.
///
/// In production the engine lives in a `static` and is never dropped: dropping it removes every
/// detour under running threads. Detoured targets are never freed, so `max_hooks` caps the distinct
/// targets for the lifetime of the process. Re-registering a removed hook in a new position can make
/// a call that was in flight when it was removed run that hook a second time.
pub struct HookEngine {
    chain: HookChain,
    targets: HashMap<u64, TargetHook>,
    max_hooks: usize,
    next_handle: u64,
}

impl HookEngine {
    /// An engine for at most `max_hooks` hooks and as many distinct targets.
    pub fn new(max_hooks: usize) -> Self {
        Self { chain: HookChain::new(max_hooks), targets: HashMap::new(), max_hooks, next_handle: 1 }
    }

    /// Number of live hooks.
    pub fn len(&self) -> usize {
        self.chain.len()
    }

    pub fn is_empty(&self) -> bool {
        self.chain.is_empty()
    }

    /// Adds a hook and returns its handle.
    ///
    /// # Safety
    /// `target` must be the start of a function that can be detoured; `detour` a function with the
    /// same signature. `original` must point to an 8-aligned `AtomicPtr<c_void>` that stays valid
    /// until the hook is removed and is read by the mod only through that atomic.
    pub unsafe fn hook(
        &mut self,
        target: *const c_void,
        detour: *const c_void,
        priority: i32,
        original: *mut *const c_void,
    ) -> Result<u64, HookError> {
        if target.is_null() || detour.is_null() || original.is_null() {
            return Err(HookError::InvalidArgument("null pointer"));
        }
        if target == detour {
            return Err(HookError::InvalidArgument("detour is the target"));
        }
        if !original.is_aligned() {
            return Err(HookError::InvalidArgument("misaligned original slot"));
        }
        let key = target as usize as u64;
        if !self.targets.contains_key(&key) && self.targets.len() >= self.max_hooks {
            return Err(ChainError::Full(self.max_hooks).into());
        }
        let handle = self.next_handle;
        let change = self.chain.add(key, handle, priority, detour as usize as u64, original as usize as u64)?;
        // SAFETY: the caller guarantees `target` and `detour`; the slots in `change` are registered
        // by earlier calls under the same guarantee.
        if let Err(error) = unsafe { self.apply(target, &change) } {
            let _ = self.chain.remove(handle);
            Self::write_slot(original as usize as u64, std::ptr::null());
            return Err(error);
        }
        self.next_handle += 1;
        Ok(handle)
    }

    /// Removes a hook. The target keeps its detour; with no hook left it runs the original.
    pub fn unhook(&mut self, handle: u64) -> Result<(), HookError> {
        let change = self.chain.remove(handle)?;
        let target = change.target as usize as *const c_void;
        // SAFETY: the target is already detoured, because the handle came from a hook on it, so
        // `apply` creates no detour; its slots were registered under the contract of `hook`.
        unsafe { self.apply(target, &change) }
    }

    /// # Safety
    /// The contract of `hook` holds for `target`, the detours and the slots of `change`.
    unsafe fn apply(&mut self, target: *const c_void, change: &ChainChange) -> Result<(), HookError> {
        let key = change.target;
        let first = change.first.map(|detour| detour as usize as *const c_void);
        let Some(hook) = self.targets.get(&key) else {
            let Some(first) = first else {
                return Ok(());
            };
            // SAFETY: the caller guarantees `target` and `first`.
            let hook = unsafe { TargetHook::new(target, first)? };
            Self::write_originals(&hook, change);
            hook.enable()?;
            self.targets.insert(key, hook);
            return Ok(());
        };
        Self::write_originals(hook, change);
        hook.point_at(first.unwrap_or_else(|| hook.trampoline()));
        Ok(())
    }

    // Last hook first, so a hook's own slot is valid before the hook in front of it can reach it.
    fn write_originals(hook: &TargetHook, change: &ChainChange) {
        for write in &change.originals {
            let value = match write.value {
                OriginalValue::Hook(detour) => detour as usize as *const c_void,
                OriginalValue::Trampoline => hook.trampoline(),
            };
            Self::write_slot(write.slot, value);
        }
    }

    fn write_slot(slot: u64, value: *const c_void) {
        // SAFETY: `slot` was checked non-null and aligned in `hook`, and the caller of `hook`
        // guarantees it stays a valid `AtomicPtr<c_void>` until the hook is removed.
        let slot = unsafe { AtomicPtr::<c_void>::from_ptr(slot as usize as *mut *mut c_void) };
        slot.store(value.cast_mut(), Ordering::Release);
    }
}
