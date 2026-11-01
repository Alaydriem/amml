use std::ffi::c_void;

use crate::GameEntries;
use crate::game::{CompoundTagRef, RawObject};

/// The elements of a `ListTag` of the game: owning pointers to `Tag` objects. Client main thread; the list must not
/// change while the wrapper is used.
pub struct NbtList<'a> {
    items: &'a [*mut c_void],
    entries: &'a GameEntries,
}

impl<'a> NbtList<'a> {
    // `Tag::getId` is the sixth virtual of `Tag` (after the destructor and four others), on every class of tag.
    const GET_ID_SLOT: u64 = 5;
    // The `Tag::Type` of a compound tag.
    const COMPOUND_ID: u8 = 10;

    pub(crate) fn new(items: &'a [*mut c_void], entries: &'a GameEntries) -> Self {
        Self { items, entries }
    }

    /// The number of elements.
    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// The element at `index` as a compound tag. `None` past the end, for a null element and for an element of
    /// another type, so a malformed list loses only the bad elements.
    #[allow(unsafe_code)]
    pub fn compound(&self, index: usize) -> Option<CompoundTagRef<'a>> {
        let element = *self.items.get(index)?;
        // SAFETY: a non-null element of a live list is a live Tag, whose first qword is its vtable.
        let get_id = unsafe { RawObject::slot(element, Self::GET_ID_SLOT) }?;
        // SAFETY: the slot holds `Tag::Type Tag::getId() const`: this in rcx, the type in al.
        let get_id: unsafe extern "C" fn(*mut c_void) -> u8 = unsafe { std::mem::transmute(get_id) };
        // SAFETY: the element is a live Tag.
        if unsafe { get_id(element) } != Self::COMPOUND_ID {
            return None;
        }
        // SAFETY: the element's type is compound, so it is a live CompoundTag.
        unsafe { CompoundTagRef::from_raw(element, self.entries) }
    }
}
