use std::ffi::c_void;

use amml_data::EntryId;

use crate::{LoaderState, SurfaceShape, WorldBlockPos};

/// `bool Block::canProvideFullSupport(uchar face) const` (26.63): rcx the block, the face in dl, the bool in al.
type SupportFn = unsafe extern "C" fn(*mut c_void, u8) -> u8;
/// `bool Block::canProvideFullSupport(uchar face, bool flag) const` (70.x): as above and the flag in r8b.
type SupportFlagFn = unsafe extern "C" fn(*mut c_void, u8, u8) -> u8;
/// `AABB BlockType::getCollisionShape(Block const&, IConstBlockSource const&, BlockPos const&,
/// optional_ref<GetCollisionShapeInterface const>) const`: rcx the type, the 24-byte box through rdx, the block in
/// r8, the block source in r9, the position and the optional reference (null: no entity) on the stack.
type CollisionFn = unsafe extern "C" fn(
    *mut c_void,
    *mut [f32; 6],
    *mut c_void,
    *mut c_void,
    *const WorldBlockPos,
    *const c_void,
) -> *mut [f32; 6];
/// `BrightnessPair BlockSource::getBrightnessPair(BlockPos const&) const`: rcx the block source, the two bytes (sky,
/// then block) through rdx, the position in r8.
type BrightnessFn = unsafe extern "C" fn(*mut c_void, *mut [u8; 2], *const WorldBlockPos) -> *mut [u8; 2];

/// The game side of the `world.scan_surfaces` area: whether a block gives full support on its top face, whether its
/// collision shape is empty, and the raw light at a block position. The calls are for the game thread. The support
/// call has the shape the build has (`SurfaceShape`); the collision shape is a virtual of the block type.
pub struct SurfaceAdapter {
    shape: SurfaceShape,
    support: usize,
    collision_slot: usize,
    brightness: usize,
    block_type_at: usize,
}

impl SurfaceAdapter {
    // The top face.
    const FACE_UP: u8 = 1;
    // The flag of the 70.x support call: the plain question.
    const NO_FLAG: u8 = 0;

    /// The adapter for the entries of the build; `None` when no shape of the support call is complete.
    pub fn new(state: &LoaderState) -> Option<Self> {
        let resolved = |name: &str| state.table().get(&EntryId(name.to_string())).is_some();
        let shape = SurfaceShape::pick(&resolved)?;
        let value = |name: &str| state.resolve(name).ok();
        let function = |name: &str| value(name).and_then(|value| usize::try_from(value).ok()).filter(|at| *at != 0);
        let slot = |name: &str| value(name).and_then(|value| usize::try_from(value).ok());
        let support = match shape {
            SurfaceShape::Support => function("Block.canProvideFullSupport")?,
            SurfaceShape::SupportFlag => function("Block.canProvideFullSupportWithFlag")?,
        };
        Some(Self {
            shape,
            support,
            collision_slot: slot("BlockType.getCollisionShape")?,
            brightness: function("BlockSource.getBrightnessPair")?,
            block_type_at: slot("Block.mBlockType")?,
        })
    }

    /// The shape of the build.
    pub fn shape(&self) -> SurfaceShape {
        self.shape
    }

    /// Whether the live `block` gives full support on its top face.
    ///
    /// # Safety
    /// `block` is a live game `Block`, called on the game thread.
    pub unsafe fn gives_full_top_support(&self, block: usize) -> bool {
        let block = std::ptr::with_exposed_provenance_mut::<c_void>(block);
        let answer = match self.shape {
            SurfaceShape::Support => {
                // SAFETY: the entry is `Block::canProvideFullSupport(uchar)`.
                let function = unsafe { std::mem::transmute::<usize, SupportFn>(self.support) };
                // SAFETY: the block is live (the caller's contract).
                unsafe { function(block, Self::FACE_UP) }
            }
            SurfaceShape::SupportFlag => {
                // SAFETY: the entry is `Block::canProvideFullSupport(uchar, bool)`.
                let function = unsafe { std::mem::transmute::<usize, SupportFlagFn>(self.support) };
                // SAFETY: the block is live (the caller's contract).
                unsafe { function(block, Self::FACE_UP, Self::NO_FLAG) }
            }
        };
        answer != 0
    }

    /// Whether the collision shape of the live `block` at `pos` is empty: the box is one point. A block with no block
    /// type, or a type with no such slot, is not empty.
    ///
    /// # Safety
    /// `block` is a live game `Block` and `source` the live block source that holds it, called on the game thread.
    pub unsafe fn has_empty_collision(&self, block: usize, source: usize, pos: &WorldBlockPos) -> bool {
        // SAFETY: `block` is live and `block_type_at` is the offset of its `mBlockType` pointer.
        let block_type = unsafe { Self::read_usize(block, self.block_type_at) };
        let Some(block_type) = block_type.filter(|block_type| *block_type != 0) else { return false };
        // SAFETY: the type of a live block is live and starts with its vtable.
        let Some(function) = (unsafe { Self::virtual_fn(block_type, self.collision_slot) }) else { return false };
        // SAFETY: the slot holds `BlockType::getCollisionShape`.
        let function = unsafe { std::mem::transmute::<usize, CollisionFn>(function) };
        let mut shape = [0.0f32; 6];
        // SAFETY: every argument lives across the call; a `BlockSource` is its own `IConstBlockSource` (its first
        // base); the box is the 24 bytes the callee fills through the hidden return pointer.
        unsafe {
            function(
                std::ptr::with_exposed_provenance_mut(block_type),
                &raw mut shape,
                std::ptr::with_exposed_provenance_mut(block),
                std::ptr::with_exposed_provenance_mut(source),
                pos,
                std::ptr::null(),
            )
        };
        shape[..3] == shape[3..]
    }

    /// The raw light at `pos` through the live block source `source`: the sky light, then the block light.
    ///
    /// # Safety
    /// `source` is a live block source, called on the game thread.
    pub unsafe fn light(&self, source: usize, pos: &WorldBlockPos) -> (u8, u8) {
        // SAFETY: the entry is `BlockSource::getBrightnessPair`.
        let function = unsafe { std::mem::transmute::<usize, BrightnessFn>(self.brightness) };
        let mut pair = [0u8; 2];
        // SAFETY: the block source is live (the caller's contract) and every argument lives across the call.
        unsafe { function(std::ptr::with_exposed_provenance_mut(source), &raw mut pair, pos) };
        (pair[0], pair[1])
    }

    unsafe fn read_usize(base: usize, offset: usize) -> Option<usize> {
        let address = base.checked_add(offset)?;
        // SAFETY: the caller guarantees `address` is inside a live game object.
        Some(unsafe { std::ptr::with_exposed_provenance::<usize>(address).read_unaligned() })
    }

    // The function in `slot` of the vtable at the start of `object`.
    unsafe fn virtual_fn(object: usize, slot: usize) -> Option<usize> {
        // SAFETY: `object` is a live game object whose first qword is its vtable.
        let vtable = unsafe { Self::read_usize(object, 0) }?;
        if vtable == 0 {
            return None;
        }
        // SAFETY: the vtable of the object; the entry keeps `slot` inside it.
        let function = unsafe { Self::read_usize(vtable, slot.checked_mul(size_of::<usize>())?) }?;
        (function != 0).then_some(function)
    }
}
