use std::mem::offset_of;

/// Mirror of the start of the game's `mce::MeshContext` (no vtable), `ScreenContext.meshContext` into a `ScreenContext`:
/// the render context, the camera, the global constant buffers, their manager and the current shader colour, each a
/// reference stored as a pointer. A prefix. `ScreenContext.camera`, `ScreenContext.constantBuffers` and
/// `ScreenContext.currentShaderColor` must sit at the second, third and fifth member (survey section 8.2).
#[repr(C)]
pub struct MeshContextLayout {
    pub render_context: *mut u8,          // 0x00 renderContext
    pub camera: *mut u8,                  // 0x08 camera
    pub constant_buffers: *mut u8,        // 0x10 constantBuffers
    pub constant_buffer_manager: *mut u8, // 0x18 constantBufferManager
    pub current_shader_color: *mut u8,    // 0x20 currentShaderColor
}

impl MeshContextLayout {
    pub const CAMERA: u64 = offset_of!(MeshContextLayout, camera) as u64;
    pub const CONSTANT_BUFFERS: u64 = offset_of!(MeshContextLayout, constant_buffers) as u64;
    pub const CURRENT_SHADER_COLOR: u64 = offset_of!(MeshContextLayout, current_shader_color) as u64;
}

const _: () = assert!(offset_of!(MeshContextLayout, camera) == 0x08);
const _: () = assert!(offset_of!(MeshContextLayout, constant_buffers) == 0x10);
const _: () = assert!(offset_of!(MeshContextLayout, current_shader_color) == 0x20);
