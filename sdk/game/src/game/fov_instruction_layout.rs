use std::mem::offset_of;

use amml_msvc::Optional;

/// Layout mirror of the game's `FovInstruction` (16 bytes). `CameraInstruction::mFieldOfView` is an
/// `Optional<FovInstructionLayout>` at +0x3C (20 bytes), so the engaged flag is at +0x4C.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct FovInstructionLayout {
    pub fov: f32,        // 0x00 mFov
    pub fov_clear: bool, // 0x04 mFovClear
    pub ease_time: f32,  // 0x08 mFovEaseTime
    pub ease_type: i32,  // 0x0C mFovEaseType (EasingType: Linear 0)
}

impl FovInstructionLayout {
    pub fn new(fov: f32, fov_clear: bool, ease_time: f32, ease_type: i32) -> Self {
        Self { fov, fov_clear, ease_time, ease_type }
    }
}

const _: () = assert!(offset_of!(FovInstructionLayout, fov_clear) == 0x04);
const _: () = assert!(offset_of!(FovInstructionLayout, ease_time) == 0x08);
const _: () = assert!(offset_of!(FovInstructionLayout, ease_type) == 0x0C);
const _: () = assert!(size_of::<FovInstructionLayout>() == 0x10);
const _: () = assert!(size_of::<Optional<FovInstructionLayout>>() == 0x14);
