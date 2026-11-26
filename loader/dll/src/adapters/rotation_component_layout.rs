/// The game's `ActorRotationComponent` from `ActorRotationComponent.mRot`: the rotation (pitch, then yaw, in degrees),
/// then the previous rotation the renderer interpolates from. The rotation setter of the game stores the same two
/// values at +0 and +8 (`mov [rax],xmm7; mov [rax+4],xmm0` ... `mov [rax+8],xmm7; mov [rax+0Ch],xmm0`).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct RotationComponentLayout {
    pub rot: [f32; 2],      // 0x00 mRot (pitch, yaw)
    pub previous: [f32; 2], // 0x08 previous rotation (pitch, yaw)
}

const _: () = assert!(size_of::<RotationComponentLayout>() == 0x10);
