use amml_msvc::{MsvcError, MsvcHeap, MsvcVec};

/// Edits a game `std::vector` in place: growth on the game heap with the MSVC STL rule (the capacity grows by half, or
/// to the size asked for when that is more), so the game's own code goes on using and freeing the vector.
pub struct GameVector;

impl GameVector {
    /// Makes room for `total` elements; the elements stay as they were. The old block, if any, is freed once after the
    /// copy.
    ///
    /// # Safety
    /// The vector's block came from `heap` (or is null), and `heap` is the game's allocator.
    pub unsafe fn reserve<T: Copy>(heap: &MsvcHeap, vec: &mut MsvcVec<T>, total: usize) -> Result<(), MsvcError> {
        let [first, _, end] = vec.raw_parts();
        let len = vec.len();
        let capacity = if first.is_null() { 0 } else { (end.addr() - first.addr()) / size_of::<T>() };
        if total <= capacity {
            return Ok(());
        }
        let grown = capacity + capacity / 2;
        let new_capacity = grown.max(total);
        let bytes =
            new_capacity.checked_mul(size_of::<T>()).ok_or(MsvcError::AllocationFailed { bytes: usize::MAX })?;
        // SAFETY: the caller guarantees the game's allocator.
        let block = unsafe { heap.allocate(bytes) }.cast::<T>();
        if block.is_null() {
            return Err(MsvcError::AllocationFailed { bytes });
        }
        // SAFETY: the new block holds `new_capacity >= len` elements and does not overlap the old one; the old block
        // came from `heap` for `capacity` elements and is not used again.
        unsafe {
            if !first.is_null() {
                std::ptr::copy_nonoverlapping(first, block, len);
                heap.deallocate(first.cast(), capacity * size_of::<T>());
            }
            vec.set_raw_parts(block, block.add(len), block.add(new_capacity));
        }
        Ok(())
    }

    /// Appends `count` elements, element `index` being `item(index)`, growing the vector when its capacity is short.
    ///
    /// # Safety
    /// As `reserve`.
    pub unsafe fn extend_with<T: Copy>(
        heap: &MsvcHeap,
        vec: &mut MsvcVec<T>,
        count: usize,
        mut item: impl FnMut(usize) -> T,
    ) -> Result<(), MsvcError> {
        if count == 0 {
            return Ok(());
        }
        let len = vec.len();
        // SAFETY: passed on from the caller.
        unsafe { Self::reserve(heap, vec, len + count) }?;
        let [first, _, end] = vec.raw_parts();
        // SAFETY: the reserve left room for `len + count` elements from `first`; the elements are `Copy`, so a write
        // drops nothing.
        unsafe {
            for index in 0..count {
                first.add(len + index).write(item(index));
            }
            vec.set_raw_parts(first, first.add(len + count), end);
        }
        Ok(())
    }
}
