use std::ffi::c_void;

/// A zeroed, 16-byte aligned buffer that stands in for a game object. Values go in at the game's offsets (the
/// values of `data/`), not at the offsets of a mirror, so a test checks the mirror against the game.
pub struct GameMemory {
    words: Vec<u128>,
}

impl GameMemory {
    pub fn new(len: usize) -> Self {
        Self { words: vec![0; len.div_ceil(16)] }
    }

    /// Writes `value` at byte `at`.
    pub fn put<T: Copy>(&mut self, at: usize, value: T) {
        assert!(at + size_of::<T>() <= self.words.len() * 16, "{at:#x} is past the end");
        // SAFETY: the bytes are inside the buffer; the write does not need alignment.
        unsafe { self.ptr().cast::<u8>().add(at).cast::<T>().write_unaligned(value) };
    }

    /// Reads the `T` at byte `at`.
    pub fn get<T: Copy>(&self, at: usize) -> T {
        assert!(at + size_of::<T>() <= self.words.len() * 16, "{at:#x} is past the end");
        // SAFETY: as `put`.
        unsafe { self.words.as_ptr().cast::<u8>().add(at).cast::<T>().read_unaligned() }
    }

    pub fn ptr(&mut self) -> *mut c_void {
        self.words.as_mut_ptr().cast()
    }
}
