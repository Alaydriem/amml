use std::mem::offset_of;

use amml_msvc::MsvcMap;

/// Layout mirror of the `CompoundTagVariant` (0x30 bytes) that a `CompoundTag` map node holds as its value: the Tag
/// object itself (its vtable first), then the variant index byte, which is the `Tag::Type` of the alternative.
#[repr(C)]
pub struct TagVariantLayout {
    // 0x00 the storage of the alternative: the Tag object, vtable first
    tag: [u64; 5],
    // 0x28 the variant index
    pub index: u8,
    _padding: [u8; 7],
}

impl TagVariantLayout {
    /// The index of a `ByteTag`.
    pub const BYTE: u8 = 1;
    /// The index of an `IntTag`.
    pub const INT: u8 = 3;
    /// The index of a `ListTag`.
    pub const LIST: u8 = 9;

    /// The offset of the index byte from the start of a map node: the `CompoundTag.nodeVariantIndex` value this
    /// mirror fits.
    pub const NODE_INDEX_OFFSET: u64 = (MsvcMap::<Self>::VALUE_OFFSET + offset_of!(Self, index)) as u64;

    /// The bytes of the storage after the Tag's vtable: the value of a scalar tag, the members of any other.
    pub fn storage_after_vtable(&self) -> &[u8] {
        let words = &self.tag[1..];
        // SAFETY: the four words after the vtable are plain memory of this struct, read as bytes for its lifetime.
        unsafe { std::slice::from_raw_parts(words.as_ptr().cast::<u8>(), std::mem::size_of_val(words)) }
    }

    /// The address of the Tag object of the alternative.
    pub fn tag(&self) -> usize {
        self.tag.as_ptr().expose_provenance()
    }
}

const _: () = assert!(TagVariantLayout::NODE_INDEX_OFFSET == 0x68);
const _: () = assert!(size_of::<TagVariantLayout>() == 0x30);
