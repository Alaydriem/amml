/// Plain reads of a game object at a member offset.
pub struct ObjectRead;

impl ObjectRead {
    /// The value of type `T` at `base + offset`; `None` for a null base or an offset that overflows.
    ///
    /// # Safety
    /// A non-null `base` is the address of a live game object that holds a `T` at `offset`, readable for the
    /// call. The read is unaligned.
    pub(crate) unsafe fn value<T: Copy>(base: usize, offset: usize) -> Option<T> {
        if base == 0 {
            return None;
        }
        let address = base.checked_add(offset)?;
        // SAFETY: the caller guarantees the member at `address` is readable for a `T`.
        Some(unsafe { std::ptr::with_exposed_provenance::<T>(address).read_unaligned() })
    }

    /// The non-null pointer at `base + offset`.
    ///
    /// # Safety
    /// As for `value`, with a pointer member.
    pub(crate) unsafe fn pointer(base: usize, offset: usize) -> Option<usize> {
        // SAFETY: the caller's guarantee is passed on.
        unsafe { Self::value::<usize>(base, offset) }.filter(|pointer| *pointer != 0)
    }
}
