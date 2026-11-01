use crate::errors::MsvcError;
use crate::structs::heap::MsvcHeap;

/// Largest capacity that stays in the inline buffer.
const SMALL_CAPACITY: usize = 15;

/// Layout mirror of the release MSVC `std::string` (32 bytes).
///
/// While `capacity <= 15` the bytes live in `storage`. Otherwise the first 8 bytes of `storage`
/// hold a pointer to `capacity + 1` heap bytes.
#[repr(C)]
pub struct MsvcString {
    storage: [u8; 16],
    size: usize,
    capacity: usize,
}

impl MsvcString {
    fn empty() -> Self {
        Self { storage: [0; 16], size: 0, capacity: SMALL_CAPACITY }
    }

    fn data_ptr(&self) -> *const u8 {
        if self.capacity <= SMALL_CAPACITY {
            return self.storage.as_ptr();
        }
        // SAFETY: in heap mode the first 8 bytes of `storage` hold the data pointer; `storage` is 16 bytes.
        unsafe { self.storage.as_ptr().cast::<*const u8>().read_unaligned() }
    }

    pub fn as_bytes(&self) -> &[u8] {
        // SAFETY: `data_ptr` points at `size` initialised bytes that live as long as `self`.
        unsafe { std::slice::from_raw_parts(self.data_ptr(), self.size) }
    }

    pub fn to_str(&self) -> Result<&str, MsvcError> {
        Ok(std::str::from_utf8(self.as_bytes())?)
    }

    /// Builds a string whose heap block, if any, comes from the game allocator.
    ///
    /// Returns `AllocationFailed` if the heap's allocation function returns null. The MSVC `operator new`
    /// throws on failure, so only a function that returns null can cause it.
    ///
    /// # Safety
    /// The heap's function pointers must be valid `operator new` and `operator delete` entry points.
    pub unsafe fn new_in(heap: &MsvcHeap, text: &str) -> Result<MsvcString, MsvcError> {
        let bytes = text.as_bytes();
        let mut out = Self::empty();
        if bytes.len() <= SMALL_CAPACITY {
            out.storage[..bytes.len()].copy_from_slice(bytes);
            out.size = bytes.len();
            return Ok(out);
        }
        // SAFETY: the caller guarantees the heap is valid.
        let ptr = unsafe { heap.allocate(bytes.len() + 1) };
        if ptr.is_null() {
            return Err(MsvcError::AllocationFailed { bytes: bytes.len() + 1 });
        }
        // SAFETY: `ptr` points to `len + 1` writable bytes just allocated.
        unsafe {
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), ptr, bytes.len());
            ptr.add(bytes.len()).write(0);
        }
        // SAFETY: `storage` is 16 bytes, room for one pointer at offset 0.
        unsafe { out.storage.as_mut_ptr().cast::<*mut u8>().write_unaligned(ptr) };
        out.size = bytes.len();
        out.capacity = bytes.len();
        Ok(out)
    }

    /// Frees the heap block, if any, and resets to an empty small string.
    ///
    /// # Safety
    /// In heap mode the block must have come from `heap` (or the C++ allocator it mirrors) for
    /// `capacity + 1` bytes and must not be used again.
    pub unsafe fn release(&mut self, heap: &MsvcHeap) {
        if self.capacity > SMALL_CAPACITY {
            let ptr = self.data_ptr().cast_mut();
            // SAFETY: the caller guarantees the block was allocated for `capacity + 1` bytes by this heap.
            unsafe { heap.deallocate(ptr, self.capacity + 1) };
        }
        *self = Self::empty();
    }
}
