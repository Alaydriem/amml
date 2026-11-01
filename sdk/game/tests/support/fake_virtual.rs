use std::ffi::c_void;

use super::GameMemory;

const SLOTS: usize = 128;

/// A fake game object with a vtable: the first qword of the memory points at a table whose slots hold the given
/// function addresses. Fields go in at the game's offsets with `put`.
pub struct FakeVirtual {
    _vtable: Box<[usize; SLOTS]>,
    memory: GameMemory,
}

impl FakeVirtual {
    /// An object of `len` bytes whose vtable has `functions` as (slot, code address) pairs.
    pub fn new(len: usize, functions: &[(u64, usize)]) -> Self {
        let mut vtable = Box::new([0usize; SLOTS]);
        for (slot, function) in functions {
            vtable[usize::try_from(*slot).unwrap()] = *function;
        }
        let mut memory = GameMemory::new(len.max(16));
        memory.put::<usize>(0, vtable.as_ptr() as usize);
        Self { _vtable: vtable, memory }
    }

    pub fn put<T: Copy>(&mut self, at: usize, value: T) {
        self.memory.put(at, value);
    }

    pub fn ptr(&mut self) -> *mut c_void {
        self.memory.ptr()
    }
}
