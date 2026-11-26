use std::collections::HashSet;
use std::ffi::c_void;
use std::sync::{Mutex, MutexGuard, OnceLock};

use amml_abi::{AmmlBlockHandle, AmmlBlockInfo, AmmlItemSnapshot, AmmlStatus, AmmlThread};
use amml_data::{AdapterCatalog, Availability, EntryId, NeedKind};

use crate::{
    BlockHandles, BlockStateAdapter, HandleStamp, InventoryAdapter, LoaderState, ObjectRead, ThreadNote, WorldAdapter,
};

static ADAPTER: OnceLock<BlockReadAdapter> = OnceLock::new();

/// `Block const& BlockSource::getBlock(BlockPos const&) const`: rcx the block source, the position in rdx.
type GetBlockFn = unsafe extern "C" fn(usize, *const [i32; 3]) -> usize;
/// `bool BlockType::canBeBuiltOver(Block const&, BlockSource const&, BlockPos const&) const`: rcx the type, rdx the block,
/// r8 the block source, r9 the position; the bool in al.
type BuiltOverFn = unsafe extern "C" fn(usize, usize, usize, *const [i32; 3]) -> u8;
/// `bool BlockType::isInteractiveBlock() const`: rcx the type; the bool in al.
type InteractiveFn = unsafe extern "C" fn(usize) -> u8;

// The calls of the area that note the thread they run on, one note each.
#[derive(Clone, Copy)]
enum Hook {
    At,
    Info,
    BuiltOver,
    Item,
    HandleOf,
}

impl Hook {
    // The game entry the call stands for, as the `world thread` line names it.
    fn entry(self) -> &'static str {
        match self {
            Self::At => "BlockSource.getBlock",
            Self::Info => "BlockType.isInteractiveBlock",
            Self::BuiltOver => "BlockType.canBeBuiltOver",
            Self::Item => "BlockType.asItemInstance",
            Self::HandleOf => "Block.mBlockType",
        }
    }
}

// The blocks given in one game frame.
struct Issued {
    frame: u64,
    blocks: HashSet<usize>,
}

/// The `world.blocks` area: the block at a position of the block source of the local player, and what is read of it.
///
/// A block handle is the game `Block` address with the low 16 bits of the game frame in its top bits (`HandleStamp`),
/// and it is accepted only for a block that was given in the frame of the call, as a chunk handle of `world.chunk_read`
/// is: a handle kept past its frame is refused and never read. The loader gives one handle per block state, so a frame
/// holds few of them.
///
/// `block_at` calls `BlockSource::getBlock` (the body behind its `$getBlock` thunk) on the block source that
/// `WorldAdapter::block_source` checked. `info` reads the type id and the air flag through the `BlockTypeReader` of the
/// chunk reads (the type is compared with the registry's air by name, no game call); only a block that is not air is
/// asked `isInteractiveBlock` through the vtable of its type. `built_over` calls the `canBeBuiltOver` slot with the block
/// source and the position it was given. `item` and `handle_of` share the item read and the interned handles of
/// `world.block_states`.
///
/// Every call runs on the game thread only (`REFUSED` otherwise) and needs the gate open (`NOT_READY`).
pub struct BlockReadAdapter {
    state: &'static LoaderState,
    get_block: GetBlockFn,
    built_over_slot: usize,
    interactive_slot: usize,
    block_type_at: usize,
    issued: Mutex<Issued>,
    threads: [ThreadNote; 5],
}

impl BlockReadAdapter {
    pub const AREA: &'static str = "world.blocks";
    const VERSION: u32 = 1;
    // The most blocks one frame gives; the game has a few thousand block states.
    const MAX_ISSUED: usize = 16384;

    /// Offers `world.blocks` when its catalog shape resolved and the chunk adapter can read block types, and logs one
    /// `area adapter` or `area not available` line. Works once per process: a second call offers nothing and gives the
    /// catalog row. Install after `WorldAdapter`.
    pub fn install(state: &'static LoaderState) -> Vec<Availability> {
        let resolved = |name: &str| state.table().get(&EntryId(name.to_string())).is_some();
        let mut rows: Vec<Availability> = AdapterCatalog::availability(&resolved)
            .into_iter()
            .filter(|row| row.kind == NeedKind::Area && row.id == Self::AREA)
            .collect();
        for row in &mut rows {
            if row.shape.is_none() {
                curia::info!("area not available", { area: row.id, missing: row.missing.join(", ") });
                return rows;
            }
            if !WorldAdapter::get().is_some_and(|world| world.block_types().is_some()) {
                row.shape = None;
                row.missing = vec!["chunk adapter"];
                curia::info!("area not available", { area: row.id, missing: "chunk adapter" });
                return rows;
            }
        }
        let Some(adapter) = Self::new(state) else {
            curia::info!("area not available", { area: Self::AREA, missing: "block read entries" });
            return rows;
        };
        if ADAPTER.set(adapter).is_err() {
            curia::warn!("the block read adapter is already installed");
            return rows;
        }
        state.areas().offer(Self::AREA, Self::VERSION);
        curia::info!("area adapter", { area: Self::AREA, shape: "default" });
        rows
    }

    fn new(state: &'static LoaderState) -> Option<Self> {
        let value = |name: &str| state.resolve(name).ok().and_then(|value| usize::try_from(value).ok());
        let get_block = value("BlockSource.getBlock").filter(|address| *address != 0)?;
        // SAFETY: the address is the derived body of `BlockSource::getBlock`, whose signature is `GetBlockFn`.
        let get_block = unsafe { std::mem::transmute::<usize, GetBlockFn>(get_block) };
        Some(Self {
            state,
            get_block,
            built_over_slot: value("BlockType.canBeBuiltOver")?,
            interactive_slot: value("BlockType.isInteractiveBlock")?,
            block_type_at: value("Block.mBlockType")?,
            issued: Mutex::new(Issued { frame: 0, blocks: HashSet::new() }),
            threads: [const { ThreadNote::new() }; 5],
        })
    }

    /// The installed adapter; `None` before `install` or when the area was not offered.
    pub fn get() -> Option<&'static BlockReadAdapter> {
        ADAPTER.get()
    }

    /// The handle of the block at `pos` of `source`, a block source handle of the current world, for the game frame of
    /// the call; 0 when the game gives no block.
    pub fn block_at(&self, source: *const c_void, pos: [i32; 3]) -> Result<usize, AmmlStatus> {
        self.enter(Hook::At)?;
        let world = WorldAdapter::get().ok_or(AmmlStatus::NOT_AVAILABLE)?;
        let source = world.block_source(Self::AREA, Self::VERSION, source)?;
        // SAFETY: `source` is the live block source of the local player of the primary client (checked above), `pos` a
        // live `BlockPos` of three ints, called on the game thread.
        let block = unsafe { (self.get_block)(source, &raw const pos) };
        if block == 0 {
            return Ok(0);
        }
        self.issue(block, InventoryAdapter::frame())
    }

    /// The address of the game block at `pos` of the live block source `source`; 0 when the game gives none. No handle is
    /// issued: the caller reads the block in the same call.
    ///
    /// # Safety
    /// `source` is the live block source of the local player of the primary client (`WorldAdapter::block_source` checked
    /// it), called on the game thread.
    pub(crate) unsafe fn block_address(&self, source: usize, pos: [i32; 3]) -> usize {
        // SAFETY: as the function's contract; `pos` is a live `BlockPos` of three ints.
        unsafe { (self.get_block)(source, &raw const pos) }
    }

    /// The type id, the air flag and the interactive flag of `block`, a block handle of this game frame.
    pub fn info(&self, block: *const c_void) -> Result<AmmlBlockInfo, AmmlStatus> {
        self.enter(Hook::Info)?;
        let block = self.open(block)?;
        let types = WorldAdapter::get().and_then(WorldAdapter::block_types).ok_or(AmmlStatus::NOT_AVAILABLE)?;
        // SAFETY: `block` is a live game `Block` that `open` found in this frame's issue list, on the game thread.
        let shape =
            unsafe { types.walk_block(block, self.state.events().gate().epoch().0) }.ok_or(AmmlStatus::REFUSED)?;
        let interactive = if shape.is_air { false } else { self.interactive(block)? };
        Ok(AmmlBlockInfo {
            type_id: shape.id,
            is_air: u8::from(shape.is_air),
            is_interactive: u8::from(interactive),
            _reserved: [0; 2],
        })
    }

    /// Whether the type of `block` can be built over at `pos` of `source`; both handles are of this game frame.
    pub fn built_over(&self, source: *const c_void, block: *const c_void, pos: [i32; 3]) -> Result<bool, AmmlStatus> {
        self.enter(Hook::BuiltOver)?;
        let world = WorldAdapter::get().ok_or(AmmlStatus::NOT_AVAILABLE)?;
        let source = world.block_source(Self::AREA, Self::VERSION, source)?;
        let block = self.open(block)?;
        let block_type = self.block_type(block)?;
        let function = InventoryAdapter::virtual_fn(block_type, self.built_over_slot).ok_or(AmmlStatus::FAILED)?;
        // SAFETY: the slot of `BlockType.canBeBuiltOver` in the vtable of a live block type.
        let function = unsafe { std::mem::transmute::<usize, BuiltOverFn>(function) };
        // SAFETY: a live block type, its live block, the live block source of the local player and a live position, on
        // the game thread.
        Ok(unsafe { function(block_type, block, source, &raw const pos) } != 0)
    }

    /// The fields and the type name of the item stack the game makes for `block`, a block handle of this game frame:
    /// the read of `world.block_states`. `NOT_AVAILABLE` without that area.
    pub fn item(&self, block: *const c_void) -> Result<(AmmlItemSnapshot, Option<String>), AmmlStatus> {
        self.enter(Hook::Item)?;
        let block = self.open(block)?;
        BlockStateAdapter::get().ok_or(AmmlStatus::NOT_AVAILABLE)?.item_of(block)
    }

    /// `block`, a block handle of this game frame, as an owned handle of the mod at `owner` in the current world.
    pub fn handle_of(&self, owner: usize, block: *const c_void) -> Result<AmmlBlockHandle, AmmlStatus> {
        self.enter(Hook::HandleOf)?;
        let block = self.open(block)?;
        BlockHandles::handle(self.state, owner, block)
    }

    // The gate, the area and the game thread; logs the thread of the calls of `hook` once and on a change.
    fn enter(&self, hook: Hook) -> Result<(), AmmlStatus> {
        let gate = self.state.events().gate();
        gate.require_open()?;
        if !self.state.areas().is_available(Self::AREA, Self::VERSION) {
            return Err(AmmlStatus::NOT_AVAILABLE);
        }
        let on_game_thread = gate.on_game_thread();
        let thread = if on_game_thread { AmmlThread::GAME } else { AmmlThread::RENDER };
        if self.threads[hook as usize].changed(thread) {
            curia::info!("world thread", { hook: hook.entry(), game: on_game_thread });
        }
        if on_game_thread { Ok(()) } else { Err(AmmlStatus::REFUSED) }
    }

    // The handle of `block` for the game frame `frame`, and the block noted as given in it.
    fn issue(&self, block: usize, frame: u64) -> Result<usize, AmmlStatus> {
        let handle = HandleStamp::seal(block, frame).ok_or(AmmlStatus::FAILED)?;
        let mut issued = self.lock();
        if issued.frame != frame {
            issued.frame = frame;
            issued.blocks.clear();
        }
        if issued.blocks.len() >= Self::MAX_ISSUED && !issued.blocks.contains(&block) {
            return Err(AmmlStatus::FAILED);
        }
        issued.blocks.insert(block);
        Ok(handle)
    }

    // The address of the block `handle` names, when it was given in the current game frame; `REFUSED` otherwise.
    fn open(&self, handle: *const c_void) -> Result<usize, AmmlStatus> {
        let frame = InventoryAdapter::frame();
        let block = HandleStamp::open(handle.expose_provenance(), frame).ok_or(AmmlStatus::REFUSED)?;
        let issued = self.lock();
        if issued.frame == frame && issued.blocks.contains(&block) { Ok(block) } else { Err(AmmlStatus::REFUSED) }
    }

    fn block_type(&self, block: usize) -> Result<usize, AmmlStatus> {
        // SAFETY: `block` is a live block `open` gave; `mBlockType` is a pointer member.
        unsafe { ObjectRead::pointer(block, self.block_type_at) }.ok_or(AmmlStatus::FAILED)
    }

    // `isInteractiveBlock` of the type of the live `block`.
    fn interactive(&self, block: usize) -> Result<bool, AmmlStatus> {
        let block_type = self.block_type(block)?;
        let function = InventoryAdapter::virtual_fn(block_type, self.interactive_slot).ok_or(AmmlStatus::FAILED)?;
        // SAFETY: the slot of `BlockType.isInteractiveBlock` in the vtable of a live block type.
        let function = unsafe { std::mem::transmute::<usize, InteractiveFn>(function) };
        // SAFETY: a live block type, on the game thread.
        Ok(unsafe { function(block_type) } != 0)
    }

    fn lock(&self) -> MutexGuard<'_, Issued> {
        self.issued.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}
