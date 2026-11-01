use std::ffi::c_void;

/// Layout mirror of the release MSVC `std::deque<T>`: the container proxy, the block map, the map size, the offset
/// of the first element and the element count.
///
/// The elements sit in blocks of `BLOCK_SIZE` elements each; the map holds one pointer per block and its size is a
/// power of two. The mirror only reads: it never grows, shrinks or frees a deque.
#[repr(C)]
pub struct MsvcDeque<T> {
    proxy: *mut c_void,
    map: *mut *mut T,
    map_size: usize,
    offset: usize,
    size: usize,
}

impl<T> MsvcDeque<T> {
    /// Elements per block, as the MSVC `deque` picks it from the element size.
    pub const BLOCK_SIZE: usize = match size_of::<T>() {
        0..=1 => 16,
        2 => 8,
        3..=4 => 4,
        5..=8 => 2,
        _ => 1,
    };

    pub fn len(&self) -> usize {
        self.size
    }

    pub fn is_empty(&self) -> bool {
        self.size == 0
    }

    /// The address of the last element (`back()`); `None` for an empty deque or a null map.
    ///
    /// # Safety
    /// `self` must be a live `std::deque<T>` of the release MSVC library.
    pub unsafe fn back(&self) -> Option<*mut T> {
        if self.size == 0 || self.map.is_null() || self.map_size == 0 {
            return None;
        }
        let index = self.offset.checked_add(self.size - 1)?;
        let block = (index / Self::BLOCK_SIZE) & (self.map_size - 1);
        // SAFETY: the caller guarantees a live deque, whose map holds `map_size` block pointers.
        let first = unsafe { *self.map.add(block) };
        if first.is_null() {
            return None;
        }
        Some(first.wrapping_add(index % Self::BLOCK_SIZE))
    }
}
