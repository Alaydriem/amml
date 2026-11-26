use std::mem::offset_of;

/// Mirror of the game's `MatrixStack::MatrixStackRef` (16 bytes, survey section 8.2): the stack it pushed, then the
/// pushed matrix, a `glm::mat4` stored by columns. `MatrixStack::push` returns it through the hidden pointer, and its
/// destructor (`MatrixStackRef.dtor`) pops the matrix.
#[repr(C)]
pub struct MatrixStackRefLayout {
    pub stack: *mut u8,             // 0x00 stack
    pub matrix: *mut [[f32; 4]; 4], // 0x08 mat
}

const _: () = assert!(offset_of!(MatrixStackRefLayout, matrix) == 0x08);
const _: () = assert!(size_of::<MatrixStackRefLayout>() == 16);
