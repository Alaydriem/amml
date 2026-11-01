use std::ffi::c_void;
use std::mem::offset_of;

/// Layout mirror of the game's `ItemInstance` (128 bytes). `ItemInstance` adds no members to `ItemStackBase`, so this
/// is also the `ItemStackBase` part of an `ItemStack`.
#[repr(C)]
pub struct ItemInstanceLayout {
    _vtable: *const c_void,             // 0x00
    item_counter: *const *const c_void, // 0x08 mItem (WeakPtr<Item>: one SharedCounter<Item> pointer)
    pub user_data: *mut c_void,         // 0x10 mUserData (unique_ptr<CompoundTag>: the raw pointer)
    _block: *const c_void,              // 0x18 mBlock
    pub aux_value: i16,                 // 0x20 mAuxValue
    pub count: u8,                      // 0x22 mCount
    _members: [u8; 0x5D],               // 0x23 the flags, mPickupTime, mCanPlaceOn .. mChargedItem
}

impl ItemInstanceLayout {
    /// The `Item` that `mItem` points to, as `WeakPtr<Item>::get` reads it: null when there is no counter, else the
    /// counter's first member.
    ///
    /// # Safety
    /// `mItem` must be null or point to a live `SharedCounter<Item>`.
    #[allow(unsafe_code)]
    pub unsafe fn item(&self) -> *const c_void {
        if self.item_counter.is_null() {
            return std::ptr::null();
        }
        // SAFETY: the caller guarantees a non-null counter is live; the Item pointer is its first member.
        unsafe { *self.item_counter }
    }
}

const _: () = assert!(offset_of!(ItemInstanceLayout, item_counter) == 0x08);
const _: () = assert!(offset_of!(ItemInstanceLayout, user_data) == 0x10);
const _: () = assert!(offset_of!(ItemInstanceLayout, aux_value) == 0x20);
const _: () = assert!(offset_of!(ItemInstanceLayout, count) == 0x22);
const _: () = assert!(size_of::<ItemInstanceLayout>() == 0x80);
