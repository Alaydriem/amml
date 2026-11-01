use std::ffi::c_void;
use std::ptr::NonNull;

use crate::game::{CompoundTagLayout, CompoundTagVariantLayout, ListTagLayout, NbtList, RawObject};
use crate::{EntryId, GameEntries};

/// A `CompoundTag*` of the game, read by walking its `std::map` of tags: no game call, no allocation and no
/// throw. Client main thread; the reads are not thread-checked, and the tag must not change while a read runs.
pub struct CompoundTagRef<'a> {
    ptr: NonNull<c_void>,
    entries: &'a GameEntries,
}

impl<'a> CompoundTagRef<'a> {
    /// The feature of the item NBT reads: the mirror anchor, `ItemStack::fromTag` and the item name offset.
    pub const FEATURE: &'static str = "game.item_nbt";

    /// The entries of `FEATURE`.
    pub const ENTRIES: [EntryId; 3] =
        [EntryId::COMPOUND_TAG_NODE_VARIANT_INDEX, EntryId::ITEM_STACK_FROM_TAG, EntryId::ITEM_FULL_NAME];

    /// Wraps `ptr`; `None` for a null pointer, when the `CompoundTag.nodeVariantIndex` entry is off, or when its
    /// value is not where the mirror has the index byte (the layout is another one).
    ///
    /// # Safety
    /// `ptr` must be null or a live `CompoundTag` of the game build `entries` was resolved for, and must stay live
    /// while the wrapper is used.
    #[allow(unsafe_code)]
    pub unsafe fn from_raw(ptr: *mut c_void, entries: &'a GameEntries) -> Option<Self> {
        if entries.get(EntryId::COMPOUND_TAG_NODE_VARIANT_INDEX)? != CompoundTagVariantLayout::NODE_INDEX_OFFSET {
            return None;
        }
        NonNull::new(ptr).map(|ptr| Self { ptr, entries })
    }

    /// The wrapped pointer.
    pub fn as_ptr(&self) -> *mut c_void {
        self.ptr.as_ptr()
    }

    /// The entries the wrapper was made with.
    pub fn entries(&self) -> &'a GameEntries {
        self.entries
    }

    // The value of `key` with the variant index `index`.
    #[allow(unsafe_code)]
    fn value(&self, key: &str, index: u8) -> Option<&CompoundTagVariantLayout> {
        // SAFETY: `from_raw` guarantees a live CompoundTag, which the mirror fits.
        let tag = unsafe { &*self.ptr.as_ptr().cast::<CompoundTagLayout>() };
        // SAFETY: the tag's map is a live std::map<std::string, CompoundTagVariant>.
        let value = unsafe { tag.tags.find(key) }?;
        (value.index == index).then_some(value)
    }

    /// The list tag under `key`. `None` for a missing key and for a tag of another type.
    #[allow(unsafe_code)]
    pub fn list(&self, key: &str) -> Option<NbtList<'a>> {
        let value = self.value(key, CompoundTagVariantLayout::LIST)?;
        // SAFETY: the variant index says the Tag object is a ListTag, which the mirror fits.
        let list = unsafe { &*value.tag().cast::<ListTagLayout>() };
        let elements = list.items.as_slice();
        // SAFETY: the elements live as long as the tag does, which the caller of `from_raw` keeps live.
        let items = unsafe { std::slice::from_raw_parts(elements.as_ptr(), elements.len()) };
        Some(NbtList::new(items, self.entries))
    }

    /// The byte tag under `key`. `None` for a missing key and for a tag of another type.
    #[allow(unsafe_code)]
    pub fn byte(&self, key: &str) -> Option<i8> {
        let value = self.value(key, CompoundTagVariantLayout::BYTE)?;
        // SAFETY: the variant index says the Tag object is a ByteTag, whose value is the byte after the vtable.
        unsafe { RawObject::read::<i8>(value.tag(), 8) }
    }

    /// The int tag under `key`. `None` for a missing key and for a tag of another type.
    #[allow(unsafe_code)]
    pub fn int(&self, key: &str) -> Option<i32> {
        let value = self.value(key, CompoundTagVariantLayout::INT)?;
        // SAFETY: the variant index says the Tag object is an IntTag, whose value is the i32 after the vtable.
        unsafe { RawObject::read::<i32>(value.tag(), 8) }
    }
}
