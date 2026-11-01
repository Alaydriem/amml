use std::mem::offset_of;

/// Layout mirror of the game's `CaretMeasureData` (8 bytes).
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C)]
pub struct CaretMeasureDataLayout {
    pub position: i32,       // 0x00 position
    pub should_render: bool, // 0x04 shouldRender
    _padding: [u8; 3],       // 0x05
}

impl CaretMeasureDataLayout {
    /// No caret.
    pub fn none() -> Self {
        Self { position: 0, should_render: false, _padding: [0; 3] }
    }
}

const _: () = assert!(offset_of!(CaretMeasureDataLayout, should_render) == 0x04);
const _: () = assert!(size_of::<CaretMeasureDataLayout>() == 8);
