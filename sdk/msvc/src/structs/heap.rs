/// Smallest request that the MSVC STL allocates with the over-aligned "big allocation" layout.
const BIG_ALLOCATION_THRESHOLD: usize = 4096;
/// Alignment of big allocations.
const BIG_ALLOCATION_ALIGNMENT: usize = 32;
/// Extra bytes a big allocation requests (release STL: `sizeof(void*) + 32 - 1`).
const NON_USER_SIZE: usize = size_of::<usize>() + BIG_ALLOCATION_ALIGNMENT - 1;

/// The game's `operator new` and sized `operator delete`, used the way the MSVC STL allocator uses them.
#[derive(Clone, Copy)]
pub struct MsvcHeap {
    new: unsafe extern "C" fn(usize) -> *mut u8,
    delete: unsafe extern "C" fn(*mut u8, usize),
}

impl MsvcHeap {
    pub fn new(new: unsafe extern "C" fn(usize) -> *mut u8, delete: unsafe extern "C" fn(*mut u8, usize)) -> Self {
        Self { new, delete }
    }

    /// The user pointer the STL derives from the raw block address of a big allocation.
    pub fn aligned_from_raw(raw: usize) -> usize {
        raw.wrapping_add(NON_USER_SIZE) & !(BIG_ALLOCATION_ALIGNMENT - 1)
    }

    /// Allocates like `std::allocator<char>::allocate`. Returns null for zero bytes, and
    /// when the supplied allocation function returns null. The MSVC `operator new` throws on failure, so
    /// `MsvcHeap` reports a null only if the function it was given returns one.
    ///
    /// # Safety
    /// The heap's function pointers must be valid `operator new` and `operator delete` entry points.
    pub unsafe fn allocate(&self, bytes: usize) -> *mut u8 {
        if bytes == 0 {
            return std::ptr::null_mut();
        }
        if bytes < BIG_ALLOCATION_THRESHOLD {
            // SAFETY: the caller guarantees `new` is a valid operator new.
            return unsafe { (self.new)(bytes) };
        }
        let Some(block) = bytes.checked_add(NON_USER_SIZE) else {
            return std::ptr::null_mut();
        };
        // SAFETY: the caller guarantees `new` is a valid operator new.
        let raw = unsafe { (self.new)(block) };
        if raw.is_null() {
            return raw;
        }
        let aligned = Self::aligned_from_raw(raw as usize);
        let ptr = raw.wrapping_add(aligned - raw as usize);
        // SAFETY: `aligned - raw >= size_of::<usize>()` and `aligned + bytes <= raw + block`, so the
        // slot before `ptr` and the user bytes lie inside the block just allocated.
        unsafe { ptr.cast::<usize>().sub(1).write_unaligned(raw as usize) };
        ptr
    }

    /// Frees a block from `allocate` or from `std::allocator<char>::allocate` with the same `bytes`.
    ///
    /// # Safety
    /// `ptr` must come from an allocation of exactly `bytes` bytes by this heap's allocator and must not be used again.
    pub unsafe fn deallocate(&self, ptr: *mut u8, bytes: usize) {
        if ptr.is_null() {
            return;
        }
        if bytes < BIG_ALLOCATION_THRESHOLD {
            // SAFETY: the caller guarantees `ptr` was allocated for `bytes` by the game allocator.
            unsafe { (self.delete)(ptr, bytes) };
            return;
        }
        // SAFETY: the caller guarantees `ptr` came from a big allocation, so the 8 bytes before it are inside
        // that block and hold the raw pointer stored by the allocator.
        let raw = unsafe { ptr.cast::<usize>().sub(1).read_unaligned() } as *mut u8;
        // SAFETY: `raw` was returned by `operator new` for `bytes + NON_USER_SIZE` bytes.
        unsafe { (self.delete)(raw, bytes + NON_USER_SIZE) };
    }
}
