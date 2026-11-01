use std::mem::offset_of;

use amml_msvc::{MsvcDeque, Optional};

/// Layout mirror of the game's `MatrixStack` (64 bytes): the matrix deque (the top is `back()`), the sort origin and
/// the dirty flag that `MatrixStack::getTop` sets.
#[repr(C)]
pub struct MatrixStackLayout {
    pub stack: MsvcDeque<[[f32; 4]; 4]>, // 0x00 stack
    _sort_origin: Optional<u64>,         // 0x28 sortOrigin
    pub is_dirty: bool,                  // 0x38 _isDirty
}

const _: () = assert!(offset_of!(MatrixStackLayout, is_dirty) == 0x38);
const _: () = assert!(size_of::<MatrixStackLayout>() == 0x40);
