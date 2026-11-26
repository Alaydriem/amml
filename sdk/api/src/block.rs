use std::ffi::c_void;
use std::marker::PhantomData;

use amml_abi::{AmmlBlockHandle, AmmlBlockInfo, AmmlBlockPos, AmmlItemSnapshot, AmmlStr, AmmlWorldTable};

use crate::{BlockHandle, BlockPos, BlockTypeId, Blocks, ItemSnapshot};

/// A block of the world of the local player (`BlockSource::block`), as an opaque handle. It is valid in the game frame
/// of the call that gave it only: a later frame, another dimension or another world gives `None` on every use. The
/// reads run on the game thread; off it they give `None`.
///
/// The block keeps the block source and the position it was read from, because `can_be_built_over` asks the game with
/// both. Area `world.blocks` (`Area::WORLD_BLOCKS`).
pub struct Block<'f> {
    handle: *const c_void,
    source: *const c_void,
    pos: BlockPos,
    world: AmmlWorldTable,
    module: &'static str,
    _frame: PhantomData<&'f ()>,
}

impl Block<'_> {
    pub(crate) fn new(
        handle: *const c_void,
        source: *const c_void,
        pos: BlockPos,
        world: AmmlWorldTable,
        module: &'static str,
    ) -> Self {
        Self { handle, source, pos, world, module, _frame: PhantomData }
    }

    /// The id of the type of the block, the id of `Blocks::type_name` and of `ColumnSample::shown`;
    /// `BlockTypeId::NONE` when the game gave no readable type.
    pub fn type_id(&self) -> Option<BlockTypeId> {
        self.info().map(|info| BlockTypeId(info.type_id))
    }

    /// The full name of the type of the block, such as `minecraft:oak_stairs`; read through `Blocks::type_name`, so it
    /// needs `Area::WORLD_CHUNK_READ` too and is `None` for a type with no readable name.
    pub fn name(&self) -> Option<String> {
        let id = self.type_id()?;
        Blocks::new(Some(self.world), self.module).type_name(id)
    }

    /// Whether the block is air: the loader compares its type with the registry's air and calls no game function.
    pub fn is_air(&self) -> Option<bool> {
        self.info().map(|info| info.is_air != 0)
    }

    /// Whether the type of the block is interactive (a door, a chest, a lever); false for air.
    pub fn is_interactive(&self) -> Option<bool> {
        self.info().map(|info| info.is_interactive != 0)
    }

    /// Whether the game lets a block be built over this one at the position it was read from
    /// (`BlockType::canBeBuiltOver`): tall grass, snow layers, water.
    pub fn can_be_built_over(&self) -> Option<bool> {
        let read = self.world.block_built_over?;
        let pos = AmmlBlockPos::from(self.pos);
        let mut out = 0u8;
        // SAFETY: the loader checks both handles before it reads game memory; `pos` and `out` are live locals.
        #[allow(unsafe_code)]
        let status = unsafe { read(self.source, self.handle, &raw const pos, &raw mut out) };
        status.is_ok().then_some(out != 0)
    }

    /// The item stack the game makes for the block (as the pick block does), with its type name. `None` for a refused
    /// read, and on a build where the loader cannot read item stacks.
    pub fn item(&self) -> Option<ItemSnapshot> {
        let read = self.world.block_item?;
        let mut raw = AmmlItemSnapshot::empty();
        let mut read_ok = false;
        // The name is read in the same call; a longer name is read again with a buffer of its own length.
        #[allow(unsafe_code)]
        let name = Blocks::read_text(|buffer, cap, length| {
            // SAFETY: the table function is the loader's; it writes one snapshot of its `size`, at most `cap` name
            // bytes and one length.
            let status = unsafe { read(self.handle, &raw mut raw, buffer, cap, length) };
            read_ok = status.is_ok();
            read_ok
        });
        read_ok.then(|| ItemSnapshot::from_raw(&raw, name.filter(|name| !name.is_empty())))
    }

    /// The block as an owned `BlockHandle` of this mod, which stays valid across frames in this world (a block of the
    /// same state gives an equal handle). `None` for a refused read.
    pub fn handle(&self) -> Option<BlockHandle> {
        let read = self.world.block_handle_of?;
        let mut out = AmmlBlockHandle::NONE;
        // SAFETY: the loader checks the handle before it reads game memory; the module id and `out` live for the call.
        #[allow(unsafe_code)]
        let status = unsafe { read(AmmlStr::from_str(self.module), self.handle, &raw mut out) };
        if status.is_ok() { BlockHandle::from_raw(out) } else { None }
    }

    fn info(&self) -> Option<AmmlBlockInfo> {
        let read = self.world.block_info?;
        let mut out = AmmlBlockInfo { type_id: 0, is_air: 0, is_interactive: 0, _reserved: [0; 2] };
        // SAFETY: the loader checks the handle before it reads game memory, and `out` is a live local.
        #[allow(unsafe_code)]
        let status = unsafe { read(self.handle, &raw mut out) };
        status.is_ok().then_some(out)
    }
}

impl PartialEq for Block<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.handle == other.handle
    }
}
