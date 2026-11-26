/// The game's `AABBShapeComponent` (20h bytes), reached through the pointer at `Actor.aabbShapeComponent`: the box
/// (min x, y, z, max x, y, z), then the width and height. `Actor::getAttachPos` reads the box at +8 and +14h on both
/// gate builds (the entry's recipe).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct AabbShapeLayout {
    pub aabb: [f32; 6], // 0x00 mAABB
    pub dim: [f32; 2],  // 0x18 mBBDim
}

const _: () = assert!(size_of::<AabbShapeLayout>() == 0x20);
