use amml_msvc::MsvcVariant;
use std::mem::{align_of, size_of, transmute};

#[test]
fn layout_is_the_largest_alternative_then_the_index() {
    assert_eq!(size_of::<MsvcVariant<[u64; 4]>>(), 40);
    assert_eq!(align_of::<MsvcVariant<[u64; 4]>>(), 8);
    // The index byte sits right after the 32 bytes of storage.
    let variant = MsvcVariant::new(5, [0u64; 4]);
    // SAFETY: the type is repr(C) with 32 bytes of storage and a one-byte index; the 7 padding bytes are read as
    // MaybeUninit, never as values.
    let bytes: [std::mem::MaybeUninit<u8>; 40] = unsafe { transmute(variant) };
    // SAFETY: byte 32 is the index, which `new` initialised.
    assert_eq!(unsafe { bytes[32].assume_init() }, 5);
}

#[test]
fn new_keeps_the_index_and_the_storage() {
    let variant = MsvcVariant::new(1, [7u64, 0, 0, 0]);
    assert_eq!(variant.index(), 1);
    assert_eq!(variant.storage()[0], 7);
}
