use std::ffi::c_void;
use std::mem::offset_of;

use amml_msvc::MsvcMap;

/// Layout mirror of the `CompoundTagVariant` (0x30 bytes) that a `CompoundTag` map node holds as its value: the Tag
/// object itself (its vtable first) and then the variant index byte, which is the `Tag::Type` of the alternative.
#[repr(C)]
pub struct CompoundTagVariantLayout {
    tag: [u64; 5],     // 0x00 the storage of the alternative: the Tag object, vtable first
    pub index: u8,     // 0x28 the variant index
    _padding: [u8; 7], // 0x29
}

impl CompoundTagVariantLayout {
    /// The index of a `ByteTag`.
    pub const BYTE: u8 = 1;
    /// The index of an `IntTag`.
    pub const INT: u8 = 3;
    /// The index of a `ListTag`.
    pub const LIST: u8 = 9;

    /// The offset of the index byte from the start of a map node: the `CompoundTag.nodeVariantIndex` value this
    /// mirror was written for.
    pub const NODE_INDEX_OFFSET: u64 = (MsvcMap::<Self>::VALUE_OFFSET + offset_of!(Self, index)) as u64;

    /// The Tag object of the alternative.
    pub fn tag(&self) -> *const c_void {
        self.tag.as_ptr().cast()
    }
}

const _: () = assert!(CompoundTagVariantLayout::NODE_INDEX_OFFSET == 0x68);
const _: () = assert!(size_of::<CompoundTagVariantLayout>() == 0x30);
