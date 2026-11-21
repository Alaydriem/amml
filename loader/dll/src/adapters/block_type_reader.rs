use std::collections::HashMap;
use std::sync::Mutex;

use amml_runtime::{BlockTypeBook, WalkBlock};

use crate::NameReader;

/// Reads a game `Block` for the column walk: its `BlockType` through the `Block.mBlockType` entry, the full name of
/// the type through the `BlockType.mNameInfo.mFullName` entry and the `HashedString.layout` mirror, and its
/// material through the `BlockType.mMaterialInline` entry (0: the type holds a `Material const&` as a pointer and
/// the solid flag is the bool byte at 5; 1: it holds the Material by value and the solid flag is bit 10h of the byte
/// at 1). The Material is at 28h of the `BlockType` on both layouts.
///
/// A type is read once: its id (`BlockTypeBook`), its name and its walk shape are kept by the address of the type.
/// A block with no readable type, and a type with no readable name, is air; a type with no readable material is solid.
pub struct BlockTypeReader {
    names: NameReader,
    block_type_at: usize,
    full_name_at: usize,
    material_by_value: bool,
    book: BlockTypeBook,
    // The shape of each type address and the world epoch it was read in.
    types: Mutex<HashMap<usize, (u64, WalkBlock)>>,
}

impl BlockTypeReader {
    const MATERIAL_AT: usize = 0x28;
    // The solid bool of the Material that a pointer reaches.
    const SOLID_BOOL_AT: usize = 5;
    // The flag byte of the Material held by value, and its solid bit.
    const FLAGS_AT: usize = 1;
    const SOLID_BIT: u8 = 0x10;

    const AIR: &'static str = "minecraft:air";
    const SHORT_AIR: &'static str = "air";
    const WATER: &'static str = "minecraft:water";
    const PLACEHOLDER: &'static str = "minecraft:client_request_placeholder_block";

    /// A reader for the derived values of `HashedString.layout`, `Block.mBlockType`, `BlockType.mNameInfo.mFullName`
    /// and `BlockType.mMaterialInline`; `None` for a layout value or a material value that no mirror has.
    pub fn new(layout: u64, block_type_at: u64, full_name_at: u64, material_inline: u64) -> Option<Self> {
        let material_by_value = match material_inline {
            0 => false,
            1 => true,
            _ => return None,
        };
        Some(Self {
            names: NameReader::new(layout)?,
            block_type_at: usize::try_from(block_type_at).ok()?,
            full_name_at: usize::try_from(full_name_at).ok()?,
            material_by_value,
            book: BlockTypeBook::new(),
            types: Mutex::new(HashMap::new()),
        })
    }

    /// The name of the block type `id`, as it was read when the id was made; `None` for an id never given and a
    /// name that was not readable. Reads no game memory.
    pub fn type_name(&self, id: u32) -> Option<String> {
        self.book.name(id)
    }

    /// The walk shape of the game `Block` at `block`; `None` for a null block.
    ///
    /// # Safety
    /// A non-zero `block` is a live game `Block` of the build the offsets were derived for, read on the game thread.
    /// `epoch` is the world epoch: a type address seen first in a new epoch is read again (name, id and shape).
    pub unsafe fn walk_block(&self, block: usize, epoch: u64) -> Option<WalkBlock> {
        if block == 0 {
            return None;
        }
        // SAFETY: the caller guarantees a live Block; `block_type_at` is the derived offset of its `mBlockType`
        // pointer.
        let block_type = unsafe { Self::read_usize(block, self.block_type_at) }?;
        if block_type == 0 {
            return Some(Self::unreadable());
        }
        if let Some((read_in, shape)) = self.lock().get(&block_type)
            && *read_in == epoch
        {
            return Some(*shape);
        }
        // SAFETY: `block_type` is the live BlockType of a live Block, read on the game thread.
        let shape = unsafe { self.read_type(block_type, epoch) };
        self.lock().insert(block_type, (epoch, shape));
        Some(shape)
    }

    // The shape of a block with no readable type: air, so the walk goes on through it.
    fn unreadable() -> WalkBlock {
        WalkBlock { id: BlockTypeBook::NONE, is_air: true, is_water: false, is_placeholder: false, is_solid: true }
    }

    // Reads the name and the material of the type at `block_type`, and gives it an id the first time.
    unsafe fn read_type(&self, block_type: usize, epoch: u64) -> WalkBlock {
        // SAFETY: the caller guarantees a live BlockType; `full_name_at` is the derived offset of its full name.
        let name = unsafe { self.names.read(block_type + self.full_name_at) };
        let id = self.book.id_of(block_type, epoch, || name.clone().map(String::into_bytes));
        // SAFETY: as above, for the Material of the type.
        let is_solid = unsafe { self.is_solid(block_type) };
        let text = name.as_deref();
        WalkBlock {
            id,
            is_air: text.is_none_or(|text| text == Self::AIR || text == Self::SHORT_AIR || text.is_empty()),
            is_water: text == Some(Self::WATER),
            is_placeholder: text == Some(Self::PLACEHOLDER),
            is_solid,
        }
    }

    // Whether the Material of the type is solid; a Material that cannot be reached counts as solid.
    unsafe fn is_solid(&self, block_type: usize) -> bool {
        if self.material_by_value {
            // SAFETY: the caller guarantees a live BlockType, which holds its Material by value at 28h.
            let flags = unsafe { Self::read_u8(block_type, Self::MATERIAL_AT + Self::FLAGS_AT) };
            return flags.is_none_or(|flags| flags & Self::SOLID_BIT != 0);
        }
        // SAFETY: the caller guarantees a live BlockType, which holds a pointer to its Material at 28h.
        let material = unsafe { Self::read_usize(block_type, Self::MATERIAL_AT) };
        match material {
            Some(material) if material != 0 => {
                // SAFETY: a non-null Material pointer of a live BlockType reaches a live Material.
                let solid = unsafe { Self::read_u8(material, Self::SOLID_BOOL_AT) };
                solid.is_none_or(|solid| solid != 0)
            }
            _ => true,
        }
    }

    unsafe fn read_usize(base: usize, offset: usize) -> Option<usize> {
        let address = base.checked_add(offset)?;
        // SAFETY: the caller guarantees `address` is inside a live game object and 8-aligned there.
        Some(unsafe { std::ptr::with_exposed_provenance::<usize>(address).read_unaligned() })
    }

    unsafe fn read_u8(base: usize, offset: usize) -> Option<u8> {
        let address = base.checked_add(offset)?;
        // SAFETY: the caller guarantees `address` is inside a live game object.
        Some(unsafe { std::ptr::with_exposed_provenance::<u8>(address).read() })
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, HashMap<usize, (u64, WalkBlock)>> {
        self.types.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}
