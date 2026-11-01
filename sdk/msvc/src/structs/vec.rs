use crate::errors::MsvcError;
use crate::structs::heap::MsvcHeap;

/// Layout mirror of the release MSVC `std::vector<T>` (three pointers).
#[repr(C)]
pub struct MsvcVec<T> {
    first: *mut T,
    last: *mut T,
    end: *mut T,
}

impl<T> MsvcVec<T> {
    /// Compile-time check: elements must have a size and need no over-aligned allocation.
    const LAYOUT_OK: () = assert!(
        size_of::<T>() != 0 && align_of::<T>() <= 16,
        "MsvcVec<T> needs a sized T with alignment of at most 16"
    );

    pub fn len(&self) -> usize {
        let () = Self::LAYOUT_OK;
        if self.first.is_null() {
            return 0;
        }
        // SAFETY: a non-null vector has `first <= last` inside one allocation.
        unsafe { self.last.offset_from(self.first) as usize }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn as_slice(&self) -> &[T] {
        if self.first.is_null() {
            return &[];
        }
        // SAFETY: `first..last` holds `len` initialised elements that live as long as `self`.
        unsafe { std::slice::from_raw_parts(self.first, self.len()) }
    }

    /// The three pointers as stored: first, last and end (the end of the capacity).
    pub fn raw_parts(&self) -> [*mut T; 3] {
        [self.first, self.last, self.end]
    }

    /// Repoints the vector at `first`, `last` and `end` without touching the old block or the elements.
    ///
    /// # Safety
    /// The pointers must be all null or satisfy `first <= last <= end` inside one allocation, with
    /// `first..last` holding initialised elements. The old block is not freed: its owner must keep or release it.
    /// A vector that the game frees later must point into a block that the game's allocator can free, or the
    /// game must never free it.
    pub unsafe fn set_raw_parts(&mut self, first: *mut T, last: *mut T, end: *mut T) {
        self.first = first;
        self.last = last;
        self.end = end;
    }
}

impl<T: Copy> MsvcVec<T> {
    /// Copies `items` into a block from the game allocator. Capacity equals length.
    ///
    /// Returns `AllocationFailed` if the heap's allocation function returns null. The MSVC `operator new`
    /// throws on failure, so only a function that returns null can cause it. `T` must be sized and aligned to at most 16.
    ///
    /// # Safety
    /// The heap's function pointers must be valid `operator new` and `operator delete` entry points.
    pub unsafe fn from_slice_in(heap: &MsvcHeap, items: &[T]) -> Result<MsvcVec<T>, MsvcError> {
        let () = Self::LAYOUT_OK;
        let empty = MsvcVec { first: std::ptr::null_mut(), last: std::ptr::null_mut(), end: std::ptr::null_mut() };
        let bytes = std::mem::size_of_val(items);
        if bytes == 0 {
            return Ok(empty);
        }
        // SAFETY: the caller guarantees the heap is valid.
        let ptr = unsafe { heap.allocate(bytes) }.cast::<T>();
        if ptr.is_null() {
            return Err(MsvcError::AllocationFailed { bytes });
        }
        // SAFETY: `ptr` points to room for `items.len()` elements just allocated.
        unsafe { std::ptr::copy_nonoverlapping(items.as_ptr(), ptr, items.len()) };
        Ok(MsvcVec { first: ptr, last: ptr.wrapping_add(items.len()), end: ptr.wrapping_add(items.len()) })
    }

    /// Frees the block, if any, and resets to an empty vector.
    ///
    /// # Safety
    /// The block must have come from `heap` (or the C++ allocator it mirrors) and must not be used again.
    pub unsafe fn release(&mut self, heap: &MsvcHeap) {
        let () = Self::LAYOUT_OK;
        if !self.first.is_null() {
            // SAFETY: the vector's capacity is `end - first` elements, the size it was allocated with.
            let capacity = unsafe { self.end.offset_from(self.first) } as usize;
            // SAFETY: the caller guarantees the block came from this heap for `capacity * size_of::<T>()` bytes.
            unsafe { heap.deallocate(self.first.cast::<u8>(), capacity * size_of::<T>()) };
        }
        self.first = std::ptr::null_mut();
        self.last = std::ptr::null_mut();
        self.end = std::ptr::null_mut();
    }
}
