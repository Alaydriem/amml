use std::mem::offset_of;

/// Mirror of the game's `BaseActorRenderer` (B8h bytes, survey section 8.2, the same on 26.63 and 70.26): the
/// `ActorShaderManager` base (vtable and three `MaterialPtr`), then eight `MaterialPtr` (a `shared_ptr` each). The one at
/// 78h is `mDepthTestedNameTagMat`, made from the material name `name_tag_depth_tested`. The loader makes one only to own
/// that material; the `BaseActorRenderer.size` entry must have this size.
#[repr(C)]
pub struct BaseActorRendererLayout {
    _shader_manager: [u8; 0x38],         // 0x00 ActorShaderManager
    _name_tag: [u64; 8],                 // 0x38 four MaterialPtr
    pub depth_tested_name_tag: [u64; 2], // 0x78 mDepthTestedNameTagMat
    _rest: [u64; 6],                     // 0x88 three MaterialPtr
}

impl BaseActorRendererLayout {
    /// The size, the value the `BaseActorRenderer.size` entry has for this mirror.
    pub const SIZE: u64 = size_of::<BaseActorRendererLayout>() as u64;
    /// The offset of `mDepthTestedNameTagMat`.
    pub const DEPTH_TESTED_NAME_TAG: usize = offset_of!(BaseActorRendererLayout, depth_tested_name_tag);
}

const _: () = assert!(offset_of!(BaseActorRendererLayout, depth_tested_name_tag) == 0x78);
const _: () = assert!(size_of::<BaseActorRendererLayout>() == 0xB8);
