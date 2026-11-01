use std::ffi::c_void;

use crate::{LayoutFault, NameTagCollectionLayout};

/// The three pointers of the vector inside a `NameTagRenderObjectCollection`, with the size of one element. It
/// only reads: the check says whether the list has the layout the mod expects.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NameTagLayout {
    pub first: usize,
    pub last: usize,
    pub end: usize,
    pub element_size: u64,
}

impl NameTagLayout {
    /// Reads first, last and end from the collection at `collection` through [`NameTagCollectionLayout`]. `None`
    /// for a null pointer.
    ///
    /// # Safety
    /// `collection` must be null or a live, aligned collection.
    #[allow(unsafe_code)]
    pub unsafe fn read(collection: *const c_void, element_size: u64) -> Option<Self> {
        // SAFETY: the caller guarantees a null or live, aligned collection.
        let collection = unsafe { collection.cast::<NameTagCollectionLayout>().as_ref() }?;
        let [first, last, end] = collection.name_tags.raw_parts().map(|pointer| pointer as usize);
        Some(Self { first, last, end, element_size })
    }

    /// The number of elements, or the reason the layout is not the expected one.
    pub fn count(&self) -> Result<usize, LayoutFault> {
        if self.element_size == 0 {
            return Err(LayoutFault::ZeroElementSize);
        }
        let ordered = self.first <= self.last && self.last <= self.end && (self.first != 0 || self.last == 0);
        if !ordered {
            return Err(LayoutFault::Unordered);
        }
        let bytes = u64::try_from(self.last - self.first).map_err(|_| LayoutFault::Unordered)?;
        if !bytes.is_multiple_of(self.element_size) {
            return Err(LayoutFault::Misaligned);
        }
        usize::try_from(bytes / self.element_size).map_err(|_| LayoutFault::Unordered)
    }
}
