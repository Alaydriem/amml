use std::mem::offset_of;

use crate::game::MatrixStackLayout;

/// Layout mirror of the game's `mce::Camera`, up to and with `mPosition`.
///
/// The `Camera.mForward` entry anchors the mirror: a build whose `mForward` is not at the mirror's offset has a
/// different layout, and the readers and writers that use the mirror refuse it.
#[repr(C)]
pub struct CameraLayout {
    pub view_stack: MatrixStackLayout,       // 0x000 viewMatrixStack
    _world_stack: MatrixStackLayout,         // 0x040 worldMatrixStack
    pub projection_stack: MatrixStackLayout, // 0x080 projectionMatrixStack
    pub inverse_view: [[f32; 4]; 4],         // 0x0C0 mInverseViewMatrix
    pub right: [f32; 3],                     // 0x100 mRight
    pub up: [f32; 3],                        // 0x10C mUp
    pub forward: [f32; 3],                   // 0x118 mForward
    pub position: [f32; 3],                  // 0x124 mPosition
}

impl CameraLayout {
    /// The offset of `mForward`, the value the `Camera.mForward` entry must have for this layout.
    pub const FORWARD: u64 = offset_of!(CameraLayout, forward) as u64;
}

const _: () = assert!(offset_of!(CameraLayout, projection_stack) == 0x80);
const _: () = assert!(offset_of!(CameraLayout, inverse_view) == 0xC0);
const _: () = assert!(offset_of!(CameraLayout, right) == 0x100);
const _: () = assert!(offset_of!(CameraLayout, up) == 0x10C);
const _: () = assert!(offset_of!(CameraLayout, forward) == 0x118);
const _: () = assert!(size_of::<CameraLayout>() == 0x130);
