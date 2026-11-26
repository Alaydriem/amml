use std::sync::OnceLock;
use std::sync::atomic::{AtomicU8, Ordering};

use amml_abi::{AmmlBlockHandle, AmmlItemSnapshot, AmmlStatus, AmmlThread};
use amml_data::{AdapterCatalog, Availability, EntryId, NeedKind};
use amml_runtime::{BlockTagValue, BlockTagWriter};

use crate::{
    BlockHandles, BlockStateHook, InventoryAdapter, InventoryLayout, LoaderState, NameReader, ObjectRead,
    RegistryLookup, StringByteInputLayout, TagReader, ThreadNote,
};

static ADAPTER: OnceLock<BlockStateAdapter> = OnceLock::new();

/// `NbtIo::read(IDataInput&)`: rcx the 8-byte result (the `unique_ptr<CompoundTag>`), rdx the input.
type NbtReadFn = unsafe extern "C" fn(*mut usize, *mut StringByteInputLayout) -> *mut usize;
/// `tryGetBlockFromNBT(CompoundTag const&, NbtToBlockCache*)`: rcx the 16-byte result {int state, Block*}, rdx the tag,
/// r8 the cache (null: the game's own).
type TryGetFn = unsafe extern "C" fn(*mut [u64; 2], usize, usize) -> *mut [u64; 2];
/// A destructor that takes the object in rcx: `CompoundTag::~CompoundTag`, `ItemStackBase::~ItemStackBase`.
type DtorFn = unsafe extern "C" fn(usize);
/// `transformBlock`, `transformConnectionStates`, `transformWallConnectionTypes`: rcx the block, dl the rotation, r8b
/// the mirror; the turned block or null.
type TransformFn = unsafe extern "C" fn(usize, u8, u8) -> usize;
/// `hasTransformableConnectionStates`, `hasTransformableWallStates`: rcx the block, the bool in al.
type HasFn = unsafe extern "C" fn(usize) -> u8;
/// `BlockType::asItemInstance(Block const&, BlockActor const*) const`: rcx the type, rdx the result buffer, r8 the
/// block, r9 the block actor (null).
type AsItemFn = unsafe extern "C" fn(usize, *mut u8, usize, usize) -> *mut u8;

/// The `world.block_states` area: block states as owned block handles (`BlockHandles`), found by name and states, turned
/// by quarter turns, and their names and items.
///
/// `from_state` looks the name up in the registry (`BlockTypeRegistry.instance` + 18h, `RegistryLookup`) for a state of
/// the type, copies the `version` int of that state's serialization tag (`Block.mSerializationId`, written by
/// `Block::buildSerializationId` with one version for every block of the registry), writes the block tag with
/// `BlockTagWriter`, reads it with `NbtIo.read` over a `StringByteInput` in loader memory and looks it up with
/// `tryGetBlockFromNBT`. The tag the read made is destroyed and freed (18h bytes on the game heap) on every path. The
/// game's result state does not say whether the states fit (it is 2 or 1 by a byte of the block type), so `exact` is
/// the loader's own check: every state asked for is in the found block's serialization tag with the value asked for.
///
/// Every call runs on the game thread only (`REFUSED` otherwise) and needs the gate open (`NOT_READY`); a call that
/// makes a handle needs a joined world (`NOT_READY`).
pub struct BlockStateAdapter {
    state: &'static LoaderState,
    registry: RegistryLookup,
    read: NbtReadFn,
    try_get: TryGetFn,
    tag_dtor: DtorFn,
    input_vtable: usize,
    transform: TransformFn,
    connection: TransformFn,
    wall: TransformFn,
    has_connection: HasFn,
    has_wall: HasFn,
    as_item_slot: usize,
    item_dtor: DtorFn,
    block_type_at: usize,
    full_name_at: usize,
    serialization_at: usize,
    names: NameReader,
    tags: TagReader,
    threads: [ThreadNote; 4],
    // One bit per `from_state` step whose failure was logged.
    failures_logged: AtomicU8,
}

impl BlockStateAdapter {
    pub const AREA: &'static str = "world.block_states";
    const VERSION: u32 = 1;
    // The size of a `CompoundTag`, the block `NbtIo::read` allocates for the tag it gives.
    const TAG_SIZE: usize = 0x18;
    // The size of an `ItemInstance`: its deleting destructor frees 80h bytes after `ItemStackBase::~ItemStackBase`
    // on 26.63 and 70.26. The result buffer of `asItemInstance` is `ItemStack.size` (98h), which holds it.
    const ITEM_INSTANCE_SIZE: usize = 0x80;
    // `Mirror::None`: case 0 of the game's mirror switch leaves the value as it is.
    const NO_MIRROR: u8 = 0;
    // The game's rotations: 0 none, then 1, 2 and 3 clockwise quarter turns seen from above.
    const TURNS: u8 = 4;
    // The `from_state` steps whose first failure is logged.
    const STEP_LOOKUP: u8 = 1;
    const STEP_VERSION: u8 = 1 << 1;
    const STEP_READ: u8 = 1 << 2;
    const STEP_FIND: u8 = 1 << 3;

    /// Offers `world.block_states` when its catalog shape resolved and logs one `area adapter` or `area not available`
    /// line. Works once per process: a second call offers nothing and gives the catalog row.
    pub fn install(state: &'static LoaderState) -> Vec<Availability> {
        let resolved = |name: &str| state.table().get(&EntryId(name.to_string())).is_some();
        let rows: Vec<Availability> = AdapterCatalog::availability(&resolved)
            .into_iter()
            .filter(|row| row.kind == NeedKind::Area && row.id == Self::AREA)
            .collect();
        for row in &rows {
            if row.shape.is_none() {
                curia::info!("area not available", { area: row.id, missing: row.missing.join(", ") });
                return rows;
            }
        }
        let Some(adapter) = Self::new(state) else {
            curia::info!("area not available", { area: Self::AREA, missing: "block state layout" });
            return rows;
        };
        if ADAPTER.set(adapter).is_err() {
            curia::warn!("the block state adapter is already installed");
            return rows;
        }
        state.areas().offer(Self::AREA, Self::VERSION);
        curia::info!("area adapter", { area: Self::AREA, shape: "default" });
        rows
    }

    fn new(state: &'static LoaderState) -> Option<Self> {
        let value = |name: &str| state.resolve(name).ok();
        // A field or slot value, or the absolute address of a function or global entry.
        let number = |name: &str| value(name).and_then(|value| usize::try_from(value).ok()).filter(|value| *value != 0);
        let layout = value("HashedString.layout")?;
        let read = number("NbtIo.read")?;
        let try_get = number("BlockSerializationUtils.tryGetBlockFromNBT")?;
        let tag_dtor = number("CompoundTag.dtor")?;
        let transform = number("VanillaBlockStateTransformUtils.transformBlock")?;
        let connection = number("VanillaBlockStateTransformUtils.transformConnectionStates")?;
        let wall = number("VanillaBlockStateTransformUtils.transformWallConnectionTypes")?;
        let has_connection = number("VanillaBlockStateTransformUtils.hasTransformableConnectionStates")?;
        let has_wall = number("VanillaBlockStateTransformUtils.hasTransformableWallStates")?;
        let item_dtor = number("ItemStackBase.dtor")?;
        // SAFETY: every address above is the derived function of its entry, whose signature is the type it becomes.
        unsafe {
            Some(Self {
                state,
                registry: RegistryLookup::new(state)?,
                read: std::mem::transmute::<usize, NbtReadFn>(read),
                try_get: std::mem::transmute::<usize, TryGetFn>(try_get),
                tag_dtor: std::mem::transmute::<usize, DtorFn>(tag_dtor),
                input_vtable: number("StringByteInput.vtable")?,
                transform: std::mem::transmute::<usize, TransformFn>(transform),
                connection: std::mem::transmute::<usize, TransformFn>(connection),
                wall: std::mem::transmute::<usize, TransformFn>(wall),
                has_connection: std::mem::transmute::<usize, HasFn>(has_connection),
                has_wall: std::mem::transmute::<usize, HasFn>(has_wall),
                as_item_slot: number("BlockType.asItemInstance")?,
                item_dtor: std::mem::transmute::<usize, DtorFn>(item_dtor),
                block_type_at: number("Block.mBlockType")?,
                full_name_at: number("BlockType.mNameInfo.mFullName")?,
                serialization_at: number("Block.mSerializationId")?,
                names: NameReader::new(layout)?,
                tags: TagReader::new(value("CompoundTag.nodeVariantIndex")?, number("CompoundTag.vtable")?)?,
                threads: [const { ThreadNote::new() }; 4],
                failures_logged: AtomicU8::new(0),
            })
        }
    }

    /// The installed adapter; `None` before `install` or when the area was not offered.
    pub fn get() -> Option<&'static BlockStateAdapter> {
        ADAPTER.get()
    }

    /// The block of the type `name` with `states`, as a handle of the mod at `owner`, and whether every state fit.
    /// `None` for a name the game does not know.
    pub fn from_state(
        &self,
        owner: usize,
        name: &str,
        states: &[(&str, BlockTagValue<'_>)],
    ) -> Result<Option<(AmmlBlockHandle, bool)>, AmmlStatus> {
        self.enter(BlockStateHook::FromState)?;
        if self.state.events().gate().epoch().0 == 0 {
            return Err(AmmlStatus::NOT_READY);
        }
        if name.is_empty() {
            return Ok(None);
        }
        let Some(default) =
            self.data_zero_state(name).inspect_err(|status| self.failed(Self::STEP_LOOKUP, name, *status, -1))?
        else {
            self.failed(Self::STEP_LOOKUP, name, AmmlStatus::OK, -1);
            return Ok(None);
        };
        let tag = self.serialization_tag(default)?;
        // SAFETY: `tag` is the serialization tag of a live block state of the registry, which the game never changes.
        let Some(version) = (unsafe { self.tags.int(tag, "version") }) else {
            self.failed(Self::STEP_VERSION, name, AmmlStatus::FAILED, -1);
            return Err(AmmlStatus::FAILED);
        };
        let bytes = BlockTagWriter::new().write(name, states, version).map_err(|_| AmmlStatus::INVALID_ARGUMENT)?;
        let Some(block) = self.lookup_tag(name, &bytes)? else { return Ok(None) };
        let exact = self.holds(block, states);
        Ok(Some((BlockHandles::handle(self.state, owner, block)?, exact)))
    }

    /// `block` turned by `quarters` (modulo 4) clockwise quarter turns, as a handle of the mod at `owner`.
    pub fn rotated(&self, owner: usize, block: AmmlBlockHandle, quarters: u8) -> Result<AmmlBlockHandle, AmmlStatus> {
        self.enter(BlockStateHook::Rotated)?;
        let address = BlockHandles::address(self.state, block)?;
        let turned = self.turn(address, quarters % Self::TURNS);
        BlockHandles::handle(self.state, owner, turned)
    }

    /// The full name of the block type of `block`.
    pub fn name(&self, block: AmmlBlockHandle) -> Result<String, AmmlStatus> {
        self.enter(BlockStateHook::Name)?;
        let address = BlockHandles::address(self.state, block)?;
        let block_type = self.block_type(address)?;
        // SAFETY: the block type of a live block state; its full name is a HashedString of the derived layout.
        unsafe { self.names.read(block_type.checked_add(self.full_name_at).ok_or(AmmlStatus::FAILED)?) }
            .ok_or(AmmlStatus::FAILED)
    }

    /// The fields and the type name of the item stack the game makes for `block`; `NOT_AVAILABLE` without the item
    /// stack mirror of `inventory.read`.
    pub fn item(&self, block: AmmlBlockHandle) -> Result<(AmmlItemSnapshot, Option<String>), AmmlStatus> {
        self.enter(BlockStateHook::Item)?;
        let address = BlockHandles::address(self.state, block)?;
        self.item_of(address)
    }

    /// The fields and the type name of the item stack the game makes for the live block state at `address`; the read
    /// `item` and the `world.blocks` item call share. The caller has made the gate and thread checks and holds a block
    /// that stays live for the call.
    pub(crate) fn item_of(&self, address: usize) -> Result<(AmmlItemSnapshot, Option<String>), AmmlStatus> {
        self.with_item_instance(address, |inventory, layout, stack| {
            (inventory.fields_of(stack, layout), InventoryAdapter::name_of(stack, layout))
        })
    }

    /// Runs `read` on the item instance the game makes for the live block state at `address` (the stack address, the
    /// inventory adapter and its layout), then destroys the instance: it is not valid after `read` returns. The same
    /// checks as `item_of`.
    pub(crate) fn with_item_instance<R>(
        &self,
        address: usize,
        read: impl FnOnce(&InventoryAdapter, &InventoryLayout, usize) -> R,
    ) -> Result<R, AmmlStatus> {
        let inventory = InventoryAdapter::get().ok_or(AmmlStatus::NOT_AVAILABLE)?;
        let layout = inventory.layout().ok_or(AmmlStatus::NOT_AVAILABLE)?;
        let block_type = self.block_type(address)?;
        let function = InventoryAdapter::virtual_fn(block_type, self.as_item_slot).ok_or(AmmlStatus::FAILED)?;
        // SAFETY: the slot of `BlockType.asItemInstance` in the vtable of a live block type.
        let as_item = unsafe { std::mem::transmute::<usize, AsItemFn>(function) };
        if layout.stack_size < Self::ITEM_INSTANCE_SIZE {
            return Err(AmmlStatus::FAILED);
        }
        let mut buffer = vec![0u64; layout.stack_size.div_ceil(8)];
        // SAFETY: a live block type and its block state, a zeroed buffer of at least the size of an item instance
        // (checked above), no block actor.
        unsafe { as_item(block_type, buffer.as_mut_ptr().cast::<u8>(), address, 0) };
        let stack = buffer.as_ptr().expose_provenance();
        let result = read(inventory, layout, stack);
        // SAFETY: the buffer holds the item instance the game just built; it is destroyed once and not used after.
        unsafe { (self.item_dtor)(stack) };
        Ok(result)
    }

    // The gate, then the game thread; logs the thread of the calls of `hook` once and on a change.
    fn enter(&self, hook: BlockStateHook) -> Result<(), AmmlStatus> {
        let gate = self.state.events().gate();
        gate.require_open()?;
        let on_game_thread = gate.on_game_thread();
        let thread = if on_game_thread { AmmlThread::GAME } else { AmmlThread::RENDER };
        if self.threads[hook as usize].changed(thread) {
            curia::info!("world thread", { hook: hook.entry(), game: on_game_thread });
        }
        if on_game_thread { Ok(()) } else { Err(AmmlStatus::REFUSED) }
    }

    fn block_type(&self, block: usize) -> Result<usize, AmmlStatus> {
        // SAFETY: `block` is a live block state the loader holds; mBlockType is a pointer member.
        unsafe { ObjectRead::pointer(block, self.block_type_at) }.ok_or(AmmlStatus::FAILED)
    }

    fn serialization_tag(&self, block: usize) -> Result<usize, AmmlStatus> {
        block.checked_add(self.serialization_at).ok_or(AmmlStatus::FAILED)
    }

    // The default state of the block type `name`; `None` for a name the registry does not know.
    fn data_zero_state(&self, name: &str) -> Result<Option<usize>, AmmlStatus> {
        self.registry.data_zero_state(name)
    }

    // The first failure of the `from_state` step `step` for `name`, with its status (`OK` for a `None` result) and the
    // `NBTState` that `tryGetBlockFromNBT` gave (-1 before that call); later failures of that step are not logged.
    fn failed(&self, step: u8, name: &str, status: AmmlStatus, game_state: i64) {
        if self.failures_logged.fetch_or(step, Ordering::AcqRel) & step != 0 {
            return;
        }
        let step = match step {
            Self::STEP_LOOKUP => "registry lookup",
            Self::STEP_VERSION => "version read",
            Self::STEP_READ => "NbtIo.read",
            _ => "tryGetBlockFromNBT",
        };
        curia::warn!("block from_state step failed", { step: step, name: name, status: status.0, game_state: game_state });
    }

    // The block the game finds for the block tag `bytes` of `name`; the tag it read is destroyed and freed before this
    // returns.
    fn lookup_tag(&self, name: &str, bytes: &[u8]) -> Result<Option<usize>, AmmlStatus> {
        let mut input = StringByteInputLayout::new(self.input_vtable, bytes);
        let mut tag = 0usize;
        // SAFETY: an input over `bytes`, which outlive the call, and an 8-byte result.
        unsafe { (self.read)(&mut tag, &mut input) };
        if tag == 0 {
            self.failed(Self::STEP_READ, name, AmmlStatus::FAILED, -1);
            return Err(AmmlStatus::FAILED);
        }
        let mut pair = [0u64; 2];
        // SAFETY: the live tag the read just made and a 16-byte result; no cache.
        unsafe { (self.try_get)(&mut pair, tag, 0) };
        // SAFETY: the tag is the game's, made by the read above, and is destroyed once; then its block is freed with
        // the size of a CompoundTag, as its deleting destructor does.
        unsafe {
            (self.tag_dtor)(tag);
            self.state.heap().free(std::ptr::with_exposed_provenance_mut(tag), Self::TAG_SIZE);
        }
        let block = usize::try_from(pair[1]).map_err(|_| AmmlStatus::FAILED)?;
        if block == 0 {
            // The result state is the low 32 bits of the first word.
            self.failed(Self::STEP_FIND, name, AmmlStatus::OK, i64::from(pair[0] as u32 as i32));
        }
        Ok((block != 0).then_some(block))
    }

    // Whether the serialization tag of `block` holds every state of `states` with its value.
    fn holds(&self, block: usize, states: &[(&str, BlockTagValue<'_>)]) -> bool {
        let Ok(tag) = self.serialization_tag(block) else { return false };
        // SAFETY: the serialization tag of a live block state of the registry, which the game never changes.
        let Some(held) = (unsafe { self.tags.compound_at(tag, "states") }) else { return false };
        states.iter().all(|(key, value)| {
            // SAFETY: as above, the states compound of that tag.
            unsafe {
                match value {
                    BlockTagValue::Int(int) => self.tags.int(held, key) == Some(*int),
                    BlockTagValue::Bool(flag) => self.tags.byte(held, key) == Some(i8::from(*flag)),
                    BlockTagValue::Text(text) => self.tags.string(held, key).as_deref() == Some(text.as_bytes()),
                }
            }
        })
    }

    // `block` turned by `rotation` (0 to 3) as the structure placement turns it: its states, then its connection
    // states, then its wall states; a transform that gives null keeps the block.
    fn turn(&self, block: usize, rotation: u8) -> usize {
        if rotation == 0 {
            return block;
        }
        let mut block = block;
        // SAFETY: each call gets a live block state of the registry, a rotation of 1 to 3 and no mirror.
        unsafe {
            let turned = (self.transform)(block, rotation, Self::NO_MIRROR);
            if turned != 0 {
                block = turned;
            }
            if (self.has_connection)(block) != 0 {
                let turned = (self.connection)(block, rotation, Self::NO_MIRROR);
                if turned != 0 {
                    block = turned;
                }
            }
            if (self.has_wall)(block) != 0 {
                let turned = (self.wall)(block, rotation, Self::NO_MIRROR);
                if turned != 0 {
                    block = turned;
                }
            }
        }
        block
    }
}
