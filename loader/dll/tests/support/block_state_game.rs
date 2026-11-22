use std::collections::HashMap;
use std::ffi::c_void;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use amml::{HashedStringLayout30, LoaderState, StringByteInputLayout};
use amml_data::{EntryKind, Resolved};
use amml_msvc::MsvcString;

use super::{FakeCompound, FakeValue, Harness, TestHeap};

// The offsets of the stand-in Block and BlockType.
const BLOCK_TYPE_AT: usize = 0x68;
const SERIALIZATION_AT: usize = 0xF0;
const FULL_NAME_AT: usize = 0x20;
// The size of a game `CompoundTag`, as `NbtIo.read` allocates it.
const TAG_SIZE: usize = 0x18;
// The registry the loader passes: `BlockTypeRegistry.instance` + 18h.
const REGISTRY_OFFSET: usize = 0x18;

static REGISTRY: [u64; 8] = [0; 8];
static INPUT_VTABLE: [usize; 11] = [0; 11];
// The BlockType vtable: `asItemInstance` at slot 2 (no stand-in reads the others). Filled by `add_type`.
static TYPE_VTABLE: Mutex<[usize; 3]> = Mutex::new([0; 3]);
// The offsets of the stand-in item stack (the `ItemStack` mirror of `inventory.read`) and of `Item::mFullName`.
const ITEM_AT: usize = 0x08;
const COUNT_AT: usize = 0x22;
const ITEM_FULL_NAME_AT: usize = 0x20;
// `asItemInstance` calls: (block type, buffer, block, block actor); `ItemStackBase.dtor` calls: the stack.
static AS_ITEM_CALLS: Mutex<Vec<(usize, usize, usize, usize)>> = Mutex::new(Vec::new());
static ITEM_DTORS: Mutex<Vec<usize>> = Mutex::new(Vec::new());
// The item every stand-in block gives: its memory and the counter that points to it.
static ITEM: Mutex<Option<FakeItem>> = Mutex::new(None);

// A stand-in `Item` (its memory) and the shared counter whose first member points to it, as `mItem` holds it.
type FakeItem = (Box<[u64; 16]>, Box<[usize; 1]>);

static TAGS_MADE: AtomicUsize = AtomicUsize::new(0);
static TAG_DTORS: AtomicUsize = AtomicUsize::new(0);
static FIND_NOTHING: AtomicBool = AtomicBool::new(false);
static GAME: Mutex<Option<GameData>> = Mutex::new(None);

#[derive(Clone, Debug, PartialEq, Eq)]
enum StateText {
    Int(i32),
    Byte(i8),
    Text(String),
}

// One stand-in block state: its memory (the BlockType pointer at +68h and the serialization tag at +F0h) and the
// tag the memory holds.
struct FakeBlock {
    memory: Box<[u64; 64]>,
    _tag: FakeCompound,
    states: Vec<(String, StateText)>,
}

struct FakeType {
    memory: Box<[u64; 16]>,
    name: String,
    // the first block is the default state
    blocks: Vec<FakeBlock>,
}

#[derive(Default)]
struct GameData {
    types: Vec<FakeType>,
    // the bytes each live tag was read from
    live: HashMap<usize, Vec<u8>>,
    made: Vec<usize>,
    versions: Vec<i32>,
    turns: Vec<(usize, u8, u8)>,
    rotations: HashMap<(usize, u8), usize>,
    lookups: Vec<(String, u64)>,
    // block types of another stand-in game, by address, that `asItemInstance` names its item after
    external: HashMap<usize, String>,
}

/// A stand-in block registry and NBT reader with the game signatures of the `world.block_states` calls: the
/// registry lookup by `HashedString`, `NbtIo::read` over a `StringByteInput`, `tryGetBlockFromNBT`, the
/// `CompoundTag` destructor and the transform utils. Each block state holds its own serialization tag (`name`,
/// `states`, `version`) as the game builds it.
pub struct BlockStateGame;

impl BlockStateGame {
    /// The version every stand-in block writes into its serialization tag.
    pub const VERSION: i32 = 0x0123_4567;

    /// The entries of the `world.block_states` row, pointing to the stand-ins.
    pub fn entries() -> Vec<(&'static str, Resolved)> {
        let function = |address: usize| Resolved {
            kind: EntryKind::Function,
            value: address as u64 - Harness::BASE,
            recipes_agreeing: 2,
        };
        let global = |address: usize| Resolved {
            kind: EntryKind::Global,
            value: address as u64 - Harness::BASE,
            recipes_agreeing: 2,
        };
        let field = |value: usize| Resolved { kind: EntryKind::Field, value: value as u64, recipes_agreeing: 2 };
        let slot = |value| Resolved { kind: EntryKind::VSlot, value, recipes_agreeing: 2 };
        vec![
            ("BlockTypeRegistry.instance", global(REGISTRY.as_ptr().addr() - REGISTRY_OFFSET)),
            ("BlockTypeRegistry.lookupByName", function(Self::lookup_by_name as *const () as usize)),
            ("Block.mSerializationId", field(SERIALIZATION_AT)),
            ("Block.mBlockType", field(BLOCK_TYPE_AT)),
            ("BlockType.mNameInfo.mFullName", field(FULL_NAME_AT)),
            ("HashedString.layout", field(0)),
            ("StringByteInput.vtable", global(INPUT_VTABLE.as_ptr().addr())),
            ("NbtIo.read", function(Self::nbt_read as *const () as usize)),
            ("BlockSerializationUtils.tryGetBlockFromNBT", function(Self::try_get as *const () as usize)),
            ("CompoundTag.dtor", function(Self::tag_dtor as *const () as usize)),
            ("CompoundTag.nodeVariantIndex", field(0x68)),
            ("CompoundTag.vtable", global(FakeCompound::compound_vtable())),
            ("VanillaBlockStateTransformUtils.transformBlock", function(Self::transform as *const () as usize)),
            (
                "VanillaBlockStateTransformUtils.transformWallConnectionTypes",
                function(Self::transform_none as *const () as usize),
            ),
            (
                "VanillaBlockStateTransformUtils.transformConnectionStates",
                function(Self::transform_none as *const () as usize),
            ),
            (
                "VanillaBlockStateTransformUtils.hasTransformableWallStates",
                function(Self::has_none as *const () as usize),
            ),
            (
                "VanillaBlockStateTransformUtils.hasTransformableConnectionStates",
                function(Self::has_none as *const () as usize),
            ),
            ("BlockType.asItemInstance", slot(2)),
            ("ItemStackBase.dtor", function(Self::item_dtor as *const () as usize)),
        ]
    }

    /// The entries of `inventory.read` that `InventoryAdapter` needs for its stack mirror, pointing to stand-ins; the
    /// ones the block states row has too are left out.
    pub fn inventory_entries() -> Vec<(&'static str, Resolved)> {
        let function = |address: usize| Resolved {
            kind: EntryKind::Function,
            value: address as u64 - Harness::BASE,
            recipes_agreeing: 2,
        };
        let field = |value: u64| Resolved { kind: EntryKind::Field, value, recipes_agreeing: 2 };
        let slot = |value| Resolved { kind: EntryKind::VSlot, value, recipes_agreeing: 2 };
        vec![
            ("Player.playerInventory", field(0x5e0)),
            ("PlayerInventory.mInventory", field(0xb8)),
            ("PlayerInventory.mSelected", field(0x10)),
            ("FillingContainer.mItems", field(0x198)),
            ("ItemStack.size", field(0x98)),
            ("ItemStackBase.mItem", field(ITEM_AT as u64)),
            ("ItemStackBase.mCount", field(COUNT_AT as u64)),
            ("ItemStackBase.getDamageValue", function(Self::damage as *const () as usize)),
            ("Item.getMaxDamage", slot(0x24)),
            ("Item.mFullName", field(ITEM_FULL_NAME_AT as u64)),
            ("Actor.getItemInHandSlot", function(Self::damage as *const () as usize)),
            ("ItemStackBase.mBlock", field(0x18)),
            ("Block.mDirectData.mLightEmission", field(0x10)),
            ("ItemStack.fromTag", function(Self::damage as *const () as usize)),
        ]
    }

    /// Each `asItemInstance` call: (block type, buffer, block, block actor).
    pub fn as_item_calls() -> Vec<(usize, usize, usize, usize)> {
        AS_ITEM_CALLS.lock().unwrap().clone()
    }

    /// The stack each `ItemStackBase.dtor` call destroyed.
    pub fn item_dtors() -> Vec<usize> {
        ITEM_DTORS.lock().unwrap().clone()
    }

    /// Lets `asItemInstance` build an item named `name` for the block type at `block_type`, a stand-in of another game.
    pub fn register_type_name(block_type: usize, name: &str) {
        GAME.lock().unwrap().get_or_insert_with(GameData::default).external.insert(block_type, name.to_string());
    }

    /// The address of the stand-in `asItemInstance`, for the vtable of a block type of another stand-in game.
    pub fn as_item_address() -> usize {
        Self::as_item as *const () as usize
    }

    /// The address of the block type `name`.
    pub fn block_type(name: &str) -> usize {
        let game = GAME.lock().unwrap();
        game.as_ref().unwrap().types.iter().find(|fake| fake.name == name).unwrap().memory.as_ptr().addr()
    }

    /// Adds the block type `name` with one block per state list; the first list is the default state. Each value is
    /// an int, a byte (a bool state) or a text.
    pub fn add_type(name: &str, states: &[&[(&str, FakeValue)]]) {
        let mut game = GAME.lock().unwrap();
        let data = game.get_or_insert_with(GameData::default);
        let mut memory = Box::new([0u64; 16]);
        let mut vtable = TYPE_VTABLE.lock().unwrap();
        vtable[2] = Self::as_item as *const () as usize;
        memory[0] = vtable.as_ptr().addr() as u64;
        drop(vtable);
        // SAFETY: the test heap is the Rust allocator; the name is never freed.
        let text = unsafe { MsvcString::new_in(&TestHeap::heap(), name) }.unwrap();
        // SAFETY: the type block holds a HashedString of the 30h layout at FULL_NAME_AT: the hash, then the string.
        unsafe { memory.as_mut_ptr().cast::<u8>().add(FULL_NAME_AT + 8).cast::<MsvcString>().write(text) };
        let type_address = memory.as_ptr().addr() as u64;
        let blocks = states
            .iter()
            .map(|list| {
                let states: Vec<(String, StateText)> =
                    list.iter().map(|(key, value)| ((*key).to_string(), Self::text_of(value))).collect();
                let tag_states = states.iter().map(|(key, value)| (key.as_str(), Self::value_of(value))).collect();
                let tag = FakeCompound::build(vec![
                    ("name", FakeValue::Text(name.to_string())),
                    ("states", FakeValue::Compound(FakeCompound::build(tag_states))),
                    ("version", FakeValue::Int(Self::VERSION)),
                ]);
                let mut memory = Box::new([0u64; 64]);
                memory[BLOCK_TYPE_AT / 8] = type_address;
                memory[SERIALIZATION_AT / 8..SERIALIZATION_AT / 8 + 3].copy_from_slice(&tag.object_words());
                FakeBlock { memory, _tag: tag, states }
            })
            .collect();
        data.types.push(FakeType { memory, name: name.to_string(), blocks });
    }

    /// The address of block `index` of the type `name`.
    pub fn block(name: &str, index: usize) -> usize {
        let game = GAME.lock().unwrap();
        let data = game.as_ref().unwrap();
        let found = data.types.iter().find(|fake| fake.name == name).unwrap();
        found.blocks[index].memory.as_ptr().addr()
    }

    /// `transformBlock` turns `from` by `rotation` into `to`.
    pub fn set_rotation(from: usize, rotation: u8, to: usize) {
        GAME.lock().unwrap().get_or_insert_with(GameData::default).rotations.insert((from, rotation), to);
    }

    /// Makes `tryGetBlockFromNBT` find no block for any tag.
    pub fn find_nothing() {
        FIND_NOTHING.store(true, Ordering::SeqCst);
    }

    /// (tags made by `NbtIo.read`, tags destroyed, tags still live).
    pub fn tags() -> (usize, usize, usize) {
        let live = GAME.lock().unwrap().as_ref().map_or(0, |data| data.live.len());
        (TAGS_MADE.load(Ordering::SeqCst), TAG_DTORS.load(Ordering::SeqCst), live)
    }

    /// The address of each tag `NbtIo.read` made, in order.
    pub fn made_tags() -> Vec<usize> {
        GAME.lock().unwrap().as_ref().map_or_else(Vec::new, |data| data.made.clone())
    }

    /// The `version` of each tag `tryGetBlockFromNBT` read.
    pub fn versions() -> Vec<i32> {
        GAME.lock().unwrap().as_ref().map_or_else(Vec::new, |data| data.versions.clone())
    }

    /// Each `transformBlock` call: (block, rotation, mirror).
    pub fn turns() -> Vec<(usize, u8, u8)> {
        GAME.lock().unwrap().as_ref().map_or_else(Vec::new, |data| data.turns.clone())
    }

    /// Each registry lookup: (name, the hash it carried).
    pub fn lookups() -> Vec<(String, u64)> {
        GAME.lock().unwrap().as_ref().map_or_else(Vec::new, |data| data.lookups.clone())
    }

    fn text_of(value: &FakeValue) -> StateText {
        match value {
            FakeValue::Int(int) => StateText::Int(*int),
            FakeValue::Byte(byte) => StateText::Byte(*byte),
            FakeValue::Text(text) => StateText::Text(text.clone()),
            _ => panic!("a state is an int, a byte or a text"),
        }
    }

    fn value_of(value: &StateText) -> FakeValue {
        match value {
            StateText::Int(int) => FakeValue::Int(*int),
            StateText::Byte(byte) => FakeValue::Byte(*byte),
            StateText::Text(text) => FakeValue::Text(text.clone()),
        }
    }

    // HashedString::computeHash: FNV-1 over the bytes; 0 for an empty text.
    fn hash(text: &[u8]) -> u64 {
        if text.is_empty() {
            return 0;
        }
        text.iter()
            .fold(0xcbf2_9ce4_8422_2325u64, |hash, byte| hash.wrapping_mul(0x0000_0100_0000_01b3) ^ u64::from(*byte))
    }

    // `_lookupByNameImpl(name, data, resolve, logNotFound)`: the type, and its default state only for resolve 1 (the
    // game's `Block` resolve) and data 0, as the game fills result+8 only for resolve 1; empty for an unknown name or a
    // hash that is not the game's.
    extern "C" fn lookup_by_name(
        registry: *const c_void,
        out: *mut [usize; 2],
        name: *const HashedStringLayout30,
        data_value: i32,
        resolve: i32,
        _log: u8,
    ) -> *mut [usize; 2] {
        assert_eq!(registry.addr(), REGISTRY.as_ptr().addr());
        // SAFETY: the loader passes a live HashedString of the 30h layout.
        let name = unsafe { &*name };
        let text = name.text.as_bytes().to_vec();
        let mut game = GAME.lock().unwrap();
        let data = game.get_or_insert_with(GameData::default);
        data.lookups.push((String::from_utf8_lossy(&text).into_owned(), name.hash));
        let found = data
            .types
            .iter()
            .find(|fake| fake.name.as_bytes() == text.as_slice() && name.hash == Self::hash(&text))
            .map_or([0, 0], |fake| {
                let state = if resolve == 1 && data_value == 0 { fake.blocks[0].memory.as_ptr().addr() } else { 0 };
                [fake.memory.as_ptr().addr(), state]
            });
        // SAFETY: `out` is the loader's 16-byte result.
        unsafe { out.write(found) };
        out
    }

    // `NbtIo::read(IDataInput&)`: a tag on the game heap that remembers the bytes of the input.
    extern "C" fn nbt_read(out: *mut usize, input: *mut StringByteInputLayout) -> *mut usize {
        // SAFETY: the loader passes a live input over its own bytes.
        let input = unsafe { &*input };
        assert_eq!(input.vtable.addr(), INPUT_VTABLE.as_ptr().addr());
        assert_eq!(input.index, 0);
        // SAFETY: the input covers `len` bytes the loader holds for the call.
        let bytes = unsafe { std::slice::from_raw_parts(input.data, input.len) }.to_vec();
        let tag = LoaderState::get().unwrap().heap().alloc(TAG_SIZE).addr();
        TAGS_MADE.fetch_add(1, Ordering::SeqCst);
        let mut game = GAME.lock().unwrap();
        let data = game.get_or_insert_with(GameData::default);
        data.live.insert(tag, bytes);
        data.made.push(tag);
        // SAFETY: `out` is the loader's 8-byte result.
        unsafe { out.write(tag) };
        out
    }

    // `tryGetBlockFromNBT(tag, cache)`: the block of the tag's name whose states are the default states with the
    // known keys of the tag's states replaced; {1, null} for an unknown name.
    extern "C" fn try_get(out: *mut [u64; 2], tag: usize, cache: usize) -> *mut [u64; 2] {
        assert_eq!(cache, 0);
        let mut game = GAME.lock().unwrap();
        let data = game.as_mut().unwrap();
        let bytes = data.live.get(&tag).unwrap().clone();
        let (name, states, version) = TagBytes::parse(&bytes);
        data.versions.push(version);
        let result = data.types.iter().find(|fake| fake.name == name).filter(|_| !FIND_NOTHING.load(Ordering::SeqCst));
        let pair = match result {
            None => [1, 0],
            Some(fake) => {
                let mut wanted = fake.blocks[0].states.clone();
                for (key, value) in &states {
                    if let Some(slot) = wanted.iter_mut().find(|(known, _)| known == key) {
                        slot.1 = value.clone();
                    }
                }
                let block = fake.blocks.iter().find(|block| block.states == wanted).unwrap_or(&fake.blocks[0]);
                [2, block.memory.as_ptr().addr() as u64]
            }
        };
        // SAFETY: `out` is the loader's 16-byte result.
        unsafe { out.write(pair) };
        out
    }

    extern "C" fn tag_dtor(tag: usize) {
        TAG_DTORS.fetch_add(1, Ordering::SeqCst);
        let removed = GAME.lock().unwrap().as_mut().unwrap().live.remove(&tag);
        assert!(removed.is_some(), "a tag is destroyed once");
    }

    extern "C" fn transform(block: usize, rotation: u8, mirror: u8) -> usize {
        let mut game = GAME.lock().unwrap();
        let data = game.as_mut().unwrap();
        data.turns.push((block, rotation, mirror));
        data.rotations.get(&(block, rotation)).copied().unwrap_or(0)
    }

    extern "C" fn transform_none(_block: usize, _rotation: u8, _mirror: u8) -> usize {
        0
    }

    // `asItemInstance(block, blockActor)`: an item stack of count 1 of an item named after the block type, built in the
    // caller's buffer as the game builds it in place.
    extern "C" fn as_item(block_type: usize, buffer: *mut u8, block: usize, actor: usize) -> *mut u8 {
        AS_ITEM_CALLS.lock().unwrap().push((block_type, buffer.addr(), block, actor));
        let name = {
            let game = GAME.lock().unwrap();
            let data = game.as_ref().unwrap();
            data.types
                .iter()
                .find(|fake| fake.memory.as_ptr().addr() == block_type)
                .map(|fake| fake.name.clone())
                .or_else(|| data.external.get(&block_type).cloned())
                .unwrap()
        };
        let mut item = ITEM.lock().unwrap();
        let (memory, counter) = item.get_or_insert_with(|| {
            let mut memory = Box::new([0u64; 16]);
            // SAFETY: the test heap is the Rust allocator; the name is never freed.
            let text = unsafe { MsvcString::new_in(&TestHeap::heap(), &name) }.unwrap();
            // SAFETY: the item holds a HashedString of the 30h layout at ITEM_FULL_NAME_AT: the hash, then the string.
            unsafe { memory.as_mut_ptr().cast::<u8>().add(ITEM_FULL_NAME_AT + 8).cast::<MsvcString>().write(text) };
            let counter = Box::new([memory.as_ptr().addr()]);
            (memory, counter)
        });
        let _ = memory;
        // SAFETY: the loader's buffer holds at least an item instance (80h bytes).
        unsafe {
            buffer.add(ITEM_AT).cast::<usize>().write_unaligned(counter.as_ptr().addr());
            buffer.add(COUNT_AT).write(1);
        }
        buffer
    }

    extern "C" fn item_dtor(stack: usize) {
        ITEM_DTORS.lock().unwrap().push(stack);
    }

    extern "C" fn damage(_stack: *const c_void) -> i16 {
        0
    }

    extern "C" fn has_none(_block: usize) -> u8 {
        0
    }
}

// A reader of the little-endian NBT the loader writes: the root compound with `name`, `states` and `version`.
struct TagBytes<'a> {
    bytes: &'a [u8],
    at: usize,
}

impl TagBytes<'_> {
    fn parse(bytes: &[u8]) -> (String, Vec<(String, StateText)>, i32) {
        let mut reader = TagBytes { bytes, at: 0 };
        assert_eq!(reader.byte(), 10);
        assert_eq!(reader.text(), "");
        let (mut name, mut states, mut version) = (String::new(), Vec::new(), 0);
        loop {
            let kind = reader.byte();
            if kind == 0 {
                break;
            }
            let key = reader.text();
            match (kind, key.as_str()) {
                (8, "name") => name = reader.text(),
                (3, "version") => version = reader.int(),
                (10, "states") => loop {
                    let kind = reader.byte();
                    if kind == 0 {
                        break;
                    }
                    let key = reader.text();
                    let value = match kind {
                        1 => StateText::Byte(reader.byte() as i8),
                        3 => StateText::Int(reader.int()),
                        8 => StateText::Text(reader.text()),
                        other => panic!("state tag {other}"),
                    };
                    states.push((key, value));
                },
                other => panic!("entry {other:?}"),
            }
        }
        assert_eq!(reader.at, bytes.len());
        (name, states, version)
    }

    fn byte(&mut self) -> u8 {
        self.at += 1;
        self.bytes[self.at - 1]
    }

    fn int(&mut self) -> i32 {
        self.at += 4;
        i32::from_le_bytes(self.bytes[self.at - 4..self.at].try_into().unwrap())
    }

    fn text(&mut self) -> String {
        let len = usize::from(u16::from_le_bytes([self.byte(), self.byte()]));
        self.at += len;
        String::from_utf8(self.bytes[self.at - len..self.at].to_vec()).unwrap()
    }
}
