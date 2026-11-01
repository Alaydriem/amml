use std::ffi::c_void;
use std::sync::atomic::{AtomicU64, Ordering};

use region::{Allocation, Protection};

use crate::HookError;

/// A fixed entry point that jumps to an address held in a slot.
///
/// The detour of a target points at its thunk once and never changes. Moving the first hook of the
/// target is then one aligned 8 byte store into the slot, which a thread calling the target sees
/// either before or after, never half. Code: `jmp qword ptr [rip + 2]`, two bytes of padding, then
/// the slot at offset 8.
pub struct HookThunk {
    memory: Allocation,
}

impl HookThunk {
    const JUMP: [u8; 8] = [0xFF, 0x25, 0x02, 0x00, 0x00, 0x00, 0xCC, 0xCC];
    const SLOT_OFFSET: usize = 8;

    pub fn new(first: *const c_void) -> Result<Self, HookError> {
        let mut memory = region::alloc(region::page::size(), Protection::READ_WRITE_EXECUTE)?;
        let base = memory.as_mut_ptr::<u8>();
        // SAFETY: `base` is the start of a fresh page of read/write/execute memory owned by this
        // thunk, so the first 16 bytes are valid for writes and 8-aligned at offset 8.
        unsafe { base.copy_from_nonoverlapping(Self::JUMP.as_ptr(), Self::JUMP.len()) };
        let thunk = Self { memory };
        thunk.point_at(first);
        Ok(thunk)
    }

    /// The address the detour must jump to.
    pub fn entry(&self) -> *const c_void {
        self.memory.as_ptr::<c_void>()
    }

    /// Makes the thunk jump to `address`, with one aligned 8 byte store that the jump reads whole.
    pub fn point_at(&self, address: *const c_void) {
        // SAFETY: the slot is 8 bytes at offset 8 of the page this thunk owns, 8-aligned because the
        // page is, and only ever accessed as an AtomicU64 by this type or by the jump instruction.
        let slot = unsafe { AtomicU64::from_ptr(self.memory.as_ptr::<u8>().add(Self::SLOT_OFFSET).cast_mut().cast()) };
        slot.store(address as usize as u64, Ordering::Release);
    }
}
