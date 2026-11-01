use std::ffi::c_void;
use std::ptr::NonNull;

use amml_msvc::MsvcString;

use crate::game::{
    Aabb, BlockLayout, BlockPos, BlockSource, BlockTypePointerLayout, BlockTypeValueLayout, MaterialFlagsLayout,
    RawObject,
};
use crate::{EntryId, GameEntries, MaterialLayout};

/// A `Block*` of the game.
pub struct Block<'a> {
    ptr: NonNull<c_void>,
    entries: &'a GameEntries,
}

impl<'a> Block<'a> {
    /// Every entry the reads here use. `type_name`, `is_air` and `block_type_ptr` need none (the `Block` and
    /// `BlockType` mirrors); `is_solid` needs `BlockType.mMaterialInline`, `can_provide_full_support` and
    /// `collision_shape` need their function and slot.
    pub const ENTRIES: [EntryId; 3] = [
        EntryId::BLOCK_TYPE_MATERIAL_INLINE,
        EntryId::BLOCK_CAN_PROVIDE_FULL_SUPPORT,
        EntryId::BLOCK_TYPE_GET_COLLISION_SHAPE,
    ];

    /// Wraps `ptr`; `None` for a null pointer.
    ///
    /// # Safety
    /// `ptr` must be null or a live `Block` of the game build `entries` was resolved for, and must stay live while
    /// the wrapper is used.
    #[allow(unsafe_code)]
    pub unsafe fn from_raw(ptr: *mut c_void, entries: &'a GameEntries) -> Option<Self> {
        NonNull::new(ptr).map(|ptr| Self { ptr, entries })
    }

    /// The wrapped pointer.
    pub fn as_ptr(&self) -> *mut c_void {
        self.ptr.as_ptr()
    }

    /// The block type pointer (`mBlockType`); `None` when it is null. Game thread; the read is not thread-checked.
    #[allow(unsafe_code)]
    pub fn block_type_ptr(&self) -> Option<*mut c_void> {
        let block = self.ptr.as_ptr().cast::<BlockLayout>();
        // SAFETY: `from_raw` guarantees a live Block, which `BlockLayout` mirrors up to `mBlockType`.
        let block_type = unsafe { (&raw const (*block).block_type).read_unaligned() };
        (!block_type.is_null()).then_some(block_type)
    }

    /// The full name of the block type, such as `minecraft:stone`, as a view of the game's `std::string`. No copy
    /// is made; the view is valid while the game keeps the block type, which it does until the world is left.
    /// `None` when the block type pointer is null or not aligned. Game thread; the read is not thread-checked.
    #[allow(unsafe_code)]
    pub fn type_name(&self) -> Option<&MsvcString> {
        let block_type = self.block_type_ptr()?.cast::<BlockTypePointerLayout>();
        if !block_type.is_aligned() {
            return None;
        }
        // SAFETY: a non-null block type of a live block holds its full-name `HashedString` at the same place in
        // both BlockType mirrors (E0h), and the pointer is aligned, so the `std::string` that `MsvcString` mirrors
        // is a valid reference for as long as the game keeps the block type.
        Some(unsafe { &(*block_type).full_name.text })
    }

    /// Whether the block is air: its type name is `minecraft:air`. `None` when the name is not readable (see
    /// [`Block::type_name`]). Game thread; the read is not thread-checked.
    pub fn is_air(&self) -> Option<bool> {
        Some(self.type_name()?.to_str().ok()? == "minecraft:air")
    }

    /// Whether the block gives full support on `face` (1 is the top face), through `canProvideFullSupport`. `None`
    /// when the entry is off or its address is 0. Game thread; the read is not thread-checked.
    #[allow(unsafe_code)]
    pub fn can_provide_full_support(&self, face: u8) -> Option<bool> {
        let function = usize::try_from(self.entries.get(EntryId::BLOCK_CAN_PROVIDE_FULL_SUPPORT)?).ok()?;
        if function == 0 {
            return None;
        }
        // SAFETY: the entry is the address of `bool Block::canProvideFullSupport(uchar) const`, a non-static member
        // that returns a scalar in AL, and `from_raw` guarantees a live Block.
        let call: unsafe extern "C" fn(*mut c_void, u8) -> u8 =
            unsafe { std::mem::transmute(function as *const c_void) };
        // SAFETY: the signature is as above and `this` is a live Block.
        Some(unsafe { call(self.ptr.as_ptr(), face) } != 0)
    }

    /// The collision box of the block at `pos` in `region`, through the `getCollisionShape` virtual of its block
    /// type, with no entity. `None` when an entry is off, the block type pointer is null or the vtable slot is
    /// null. Game thread; the read is not thread-checked.
    #[allow(unsafe_code)]
    pub fn collision_shape(&self, region: &BlockSource, pos: &BlockPos) -> Option<Aabb> {
        let slot = self.entries.get(EntryId::BLOCK_TYPE_GET_COLLISION_SHAPE)?;
        let block_type = self.block_type_ptr()?;
        // SAFETY: a non-null block type of a live block starts with its vtable, and the entry is the slot of
        // `getCollisionShape` in it.
        let function = unsafe { RawObject::slot(block_type, slot) }?;
        // SAFETY: the slot holds `AABB BlockType::getCollisionShape(Block const&, IConstBlockSource const&,
        // BlockPos const&, optional_ref<GetCollisionShapeInterface const>) const`. The AABB is returned through a
        // hidden pointer in RDX; the block, the region and the position follow in R8, R9 and the stack, and the
        // `optional_ref` is one pointer, null for no entity.
        let call: unsafe extern "C" fn(
            *mut c_void,
            *mut Aabb,
            *const c_void,
            *const c_void,
            *const BlockPos,
            *const c_void,
        ) -> *mut Aabb = unsafe { std::mem::transmute(function) };
        let mut shape = std::mem::MaybeUninit::<Aabb>::uninit();
        // SAFETY: the signature is as above; `shape` has the size and alignment of the 24-byte box, which the
        // callee fills; the block, the region and `pos` are live for the call. A `BlockSource` is its own
        // `IConstBlockSource` (the first base).
        unsafe {
            call(
                block_type,
                shape.as_mut_ptr(),
                self.ptr.as_ptr(),
                region.as_ptr(),
                std::ptr::from_ref(pos),
                std::ptr::null(),
            );
        }
        // SAFETY: the callee initialised the box through the hidden return pointer.
        Some(unsafe { shape.assume_init() })
    }

    /// Whether the material of the block is solid, through the `BlockType` mirror that [`MaterialLayout`] selects:
    /// the Material pointer and its solid bool (`byte != 0`), or the Material held by value and its solid bit.
    /// `None` when no layout is selected, or a pointer on the path is null. Game thread; the read is not
    /// thread-checked.
    #[allow(unsafe_code)]
    pub fn is_solid(&self) -> Option<bool> {
        let layout = MaterialLayout::of(self.entries).ok()?;
        let block_type = self.block_type_ptr()?;
        match layout {
            MaterialLayout::Pointer => {
                let block_type = block_type.cast::<BlockTypePointerLayout>();
                // SAFETY: in this layout a non-null block type of a live block holds `mMaterial`, a reference
                // stored as a pointer, where the mirror has it.
                let material = unsafe { (&raw const (*block_type).material).read_unaligned() };
                if material.is_null() {
                    return None;
                }
                // SAFETY: the pointer is the live Material of the block type, which the mirror lays out.
                Some(unsafe { (&raw const (*material).solid).read_unaligned() } != 0)
            }
            MaterialLayout::Value => {
                let block_type = block_type.cast::<BlockTypeValueLayout>();
                // SAFETY: in this layout a non-null block type of a live block holds the Material by value where
                // the mirror has it.
                let flags = unsafe { (&raw const (*block_type).material.flags).read_unaligned() };
                Some(flags & MaterialFlagsLayout::SOLID != 0)
            }
        }
    }
}
