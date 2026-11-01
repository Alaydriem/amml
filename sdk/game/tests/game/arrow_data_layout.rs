use amml_game::ArrowDataLayout;

use crate::support::GameMemory;

#[test]
fn reads_the_optional_arrow_fields_at_the_payload_offsets() {
    let mut memory = GameMemory::new(0x24);
    memory.put::<[f32; 3]>(0x00, [4.0, 5.0, 6.0]); // mEndLocation, engaged flag at 0x0C
    memory.put::<u8>(0x0C, 1);
    memory.put::<f32>(0x10, 0.4); // mArrowHeadLength, engaged flag at 0x14
    memory.put::<u8>(0x14, 1);
    memory.put::<u8>(0x20, 8); // mNumSegments, engaged flag at 0x21
    memory.put::<u8>(0x21, 1);

    // SAFETY: the buffer is 16-byte aligned, zeroed and as large as the mirror.
    let arrow = unsafe { &*memory.ptr().cast::<ArrowDataLayout>() };

    assert_eq!(arrow.end.get(), Some(&[4.0, 5.0, 6.0]));
    assert_eq!(arrow.head_length.get(), Some(&0.4));
    assert!(arrow.head_radius.get().is_none());
    assert_eq!(arrow.segments.get(), Some(&8));
}
