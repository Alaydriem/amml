use amml_game::PrimitiveShapeLayout;

use crate::support::GameMemory;

#[test]
fn reads_the_optional_fields_and_the_variant_index_at_the_payload_offsets() {
    let mut memory = GameMemory::new(0xB8);
    memory.put::<u64>(0x00, 7); // mNetworkId
    memory.put::<u8>(0x08, 5); // mShapeType value, engaged flag at 0x09
    memory.put::<u8>(0x09, 1);
    memory.put::<[f32; 3]>(0x0C, [1.0, 2.0, 3.0]); // mLocation, engaged flag at 0x18
    memory.put::<u8>(0x18, 1);
    memory.put::<[f32; 4]>(0x34, [0.1, 0.2, 0.3, 1.0]); // mColor, engaged flag at 0x44
    memory.put::<u8>(0x44, 1);
    memory.put::<i32>(0x50, 2); // mDimensionId, engaged flag at 0x54
    memory.put::<u8>(0x54, 1);
    memory.put::<u64>(0x70, 0xDEAD); // mExtraDataPayload storage
    memory.put::<u8>(0xB0, 4); // alternative index: Line

    // SAFETY: the buffer is 16-byte aligned, zeroed and as large as the mirror.
    let shape = unsafe { &*memory.ptr().cast::<PrimitiveShapeLayout>() };

    assert_eq!(shape.network_id, 7);
    assert_eq!(shape.shape_type.get(), Some(&5));
    assert_eq!(shape.location.get(), Some(&[1.0, 2.0, 3.0]));
    assert_eq!(shape.color.get(), Some(&[0.1, 0.2, 0.3, 1.0]));
    assert_eq!(shape.dimension.get(), Some(&2));
    assert!(shape.rotation.get().is_none());
    assert_eq!(shape.extra[0], 0xDEAD);
    assert_eq!(shape.extra_index, 4);
}
