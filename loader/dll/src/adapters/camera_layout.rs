use std::mem::offset_of;

use crate::MatrixStackLayout;

/// Layout mirror of the game's `mce::Camera`, up to and with `mPosition`.
///
/// The `Camera.mForward` entry anchors the mirror: a build whose `mForward` is not at the mirror's offset has another
/// layout, and the camera writer and the projection read refuse it. Survey 8.3: `inspect fields` of
/// `Camera.updateViewMatrixDependencies` gives the same offsets on 26.63 and 70.26 (reads of the three stacks, writes
/// C0h..F8h and 108h..12Ch), and `Camera.mForward` is 118h on both.
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
