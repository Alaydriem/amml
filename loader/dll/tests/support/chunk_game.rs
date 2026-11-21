use std::collections::HashMap;
use std::ffi::c_void;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicU8, AtomicU64, AtomicUsize, Ordering};

use amml_data::{EntryKind, Resolved};

use super::{ActorGame, Harness};

// The bytes of the stand-in chunk; eight-byte aligned. Large enough for the heightmaps of both layouts.
#[repr(C, align(16))]
struct ChunkBytes([AtomicU8; 0x1200]);

// The stand-in sub-chunks: `SUB_CHUNKS` of 0x68 bytes.
#[repr(C, align(16))]
struct SubChunkBytes([AtomicU8; ChunkGame::SUB_CHUNKS * 0x68]);

// One stand-in `BlockType` of 0x110 bytes: the material word at 28h and the full name `HashedString` at E0h.
#[repr(C, align(16))]
struct TypeWords([AtomicU64; 34]);

// One stand-in `Block`: the `BlockType` pointer at 68h.
#[repr(C, align(16))]
struct BlockWords([AtomicU64; 14]);

static CHUNK: ChunkBytes = ChunkBytes([const { AtomicU8::new(0) }; 0x1200]);
static SUB_CHUNKS: SubChunkBytes = SubChunkBytes([const { AtomicU8::new(0) }; ChunkGame::SUB_CHUNKS * 0x68]);
static PLACEHOLDERS: [AtomicBool; ChunkGame::SUB_CHUNKS] = [const { AtomicBool::new(false) }; ChunkGame::SUB_CHUNKS];
static TYPES: [TypeWords; KINDS] = [const { TypeWords([const { AtomicU64::new(0) }; 34]) }; KINDS];
static BLOCKS: [BlockWords; KINDS] = [const { BlockWords([const { AtomicU64::new(0) }; 14]) }; KINDS];
// The one stand-in vtable every stand-in `BlockType` points to; a test puts a function in a slot with `set_type_slot`.
static TYPE_VTABLE: [AtomicUsize; TYPE_SLOTS] = [const { AtomicUsize::new(0) }; TYPE_SLOTS];
// A stand-in Material of the pointer layout: the solid bool at 5.
static SOLID_MATERIAL: [AtomicU8; 8] = [const { AtomicU8::new(0) }; 8];
static OPEN_MATERIAL: [AtomicU8; 8] = [const { AtomicU8::new(0) }; 8];
// The texts of the names that do not fit a short string.
static NAMES: [&[u8]; KINDS] = [
    b"minecraft:air",
    b"minecraft:stone",
    b"minecraft:water",
    b"minecraft:client_request_placeholder_block",
    b"minecraft:dirt",
    b"minecraft:red_flower",
    b"minecraft:stone_slab",
];
// The block kinds, and the slots of the stand-in block type vtable (the highest slot a stand-in uses is 81h).
const KINDS: usize = 7;
const TYPE_SLOTS: usize = 0x82;
// The block kind placed at a column and y relative to the lowest block y.
type Placed = HashMap<(u8, u8, i16), usize>;
static PLACED: Mutex<Option<Placed>> = Mutex::new(None);
static ASKED: Mutex<Vec<(u8, u8, i16)>> = Mutex::new(Vec::new());

/// The block kinds of the stand-in chunk besides air, which is what is where nothing was placed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChunkKind {
    Stone = 1,
    Water = 2,
    Placeholder = 3,
    /// A solid block with no map colour component.
    Dirt = 4,
    /// A block that is not solid and has no collision.
    Flower = 5,
    /// A solid block that does not give full support on its top face.
    Slab = 6,
}

/// A stand-in `LevelChunk` for the chunk adapter: a byte buffer laid out as the layout selected by the
/// `LevelChunk.mSubChunks` value, eight sub-chunks, and `getBlock`, `isPlaceHolderSubChunk` and `getChunk` stand-ins.
/// The chunk spans the block ys `MIN_Y` up to but not including `MAX_Y`; `mMax.y` holds the highest block y, `MAX_Y - 1`. Only chunk (0, 0) exists.
pub struct ChunkGame;

impl ChunkGame {
    pub const MIN_Y: i32 = -64;
    pub const MAX_Y: i32 = 64;
    pub const SUB_CHUNKS: usize = 8;
    /// The slot of `getChunk` in the stand-in block source vtable.
    pub const GET_CHUNK_SLOT: u64 = 3;
    const LOADED: u8 = 11;

    /// The actor entries and the chunk entries for the layout whose `mSubChunks` is at `sub_chunks` (1C8h or 228h) and
    /// the material layout `material_inline`; `without` names entries to leave out. Calls `setup`.
    pub fn entries(without: &[&str], sub_chunks: u64, material_inline: u64) -> Vec<(&'static str, Resolved)> {
        Self::setup(sub_chunks, material_inline);
        let mut entries = ActorGame::entries(without);
        let function = |address: usize| (EntryKind::Function, address as u64 - Harness::BASE);
        let more = [
            ("BlockSource.getChunk", (EntryKind::VSlot, Self::GET_CHUNK_SLOT)),
            ("LevelChunk.mSubChunks", (EntryKind::Field, sub_chunks)),
            ("LevelChunk.getBlock", function(Self::get_block as *const () as usize)),
            ("SubChunk.size", (EntryKind::Field, 0x68)),
            ("SubChunk.isPlaceHolderSubChunk", function(Self::is_placeholder as *const () as usize)),
            ("BlockType.mMaterialInline", (EntryKind::Field, material_inline)),
            ("BlockType.mNameInfo.mFullName", (EntryKind::Field, 0xE0)),
            ("HashedString.layout", (EntryKind::Field, 0)),
            ("Block.mBlockType", (EntryKind::Field, 0x68)),
        ];
        for (name, (kind, value)) in more {
            if !without.contains(&name) {
                entries.push((name, Resolved { kind, value, recipes_agreeing: 2 }));
            }
        }
        entries
    }

    /// Fills the stand-in chunk for the layout whose `mSubChunks` is at `sub_chunks` (1C8h, or 228h whose heightmaps are
    /// 60h further), the block types for the material layout `material_inline`, and clears the placed blocks.
    pub fn setup(sub_chunks: u64, material_inline: u64) {
        let (heightmap, render) = if sub_chunks == 0x228 { (0xC68, 0xE68) } else { (0xC08, 0xE08) };
        let sub_chunks = usize::try_from(sub_chunks).unwrap();
        // `mMin` and `mMax` are `BlockPos` of x, y, z.
        Self::put(0x64, &Self::MIN_Y.to_le_bytes());
        Self::put(0x70, &(Self::MAX_Y - 1).to_le_bytes());
        Self::put(0xF0, &[Self::LOADED]);
        let begin = std::ptr::from_ref(&SUB_CHUNKS).addr();
        Self::put(sub_chunks, &begin.to_le_bytes());
        Self::put(sub_chunks + 8, &(begin + Self::SUB_CHUNKS * 0x68).to_le_bytes());
        Self::put(sub_chunks + 16, &(begin + Self::SUB_CHUNKS * 0x68).to_le_bytes());
        for placeholder in &PLACEHOLDERS {
            placeholder.store(false, Ordering::SeqCst);
        }
        for index in 0..256 {
            Self::put(heightmap + index * 2, &0i16.to_le_bytes());
            Self::put(render + index * 2, &0i16.to_le_bytes());
        }
        HEIGHTMAP_AT.store(heightmap as u64, Ordering::SeqCst);
        RENDER_AT.store(render as u64, Ordering::SeqCst);
        SOLID_MATERIAL[5].store(1, Ordering::SeqCst);
        for kind in 0..KINDS {
            Self::build_type(kind, material_inline);
        }
        for slot in &TYPE_VTABLE {
            slot.store(0, Ordering::SeqCst);
        }
        *PLACED.lock().unwrap() = Some(HashMap::new());
        ASKED.lock().unwrap().clear();
    }

    // The stand-in `BlockType` and `Block` of `kind`.
    fn build_type(kind: usize, material_inline: u64) {
        let solid = [ChunkKind::Stone, ChunkKind::Placeholder, ChunkKind::Dirt, ChunkKind::Slab]
            .iter()
            .any(|solid| *solid as usize == kind);
        let words = &TYPES[kind].0;
        let material = if material_inline == 1 {
            // The Material by value: the type, then the flag byte with the solid bit 10h.
            u64::from(if solid { 0x10u8 } else { 0 }) << 8
        } else {
            std::ptr::from_ref(if solid { &SOLID_MATERIAL } else { &OPEN_MATERIAL }).addr() as u64
        };
        words[5].store(material, Ordering::SeqCst);
        words[0].store(std::ptr::from_ref(&TYPE_VTABLE).addr() as u64, Ordering::SeqCst);
        // The `HashedString` at E0h: the hash, then the `std::string`: a short text in the 16 bytes in place with the
        // capacity 15, a longer text through a pointer with the capacity of its length.
        let text = NAMES[kind];
        words[28].store(0, Ordering::SeqCst);
        if text.len() < 16 {
            let mut inline = [0u8; 16];
            inline[..text.len()].copy_from_slice(text);
            words[29].store(u64::from_le_bytes(inline[..8].try_into().unwrap()), Ordering::SeqCst);
            words[30].store(u64::from_le_bytes(inline[8..].try_into().unwrap()), Ordering::SeqCst);
            words[32].store(15, Ordering::SeqCst);
        } else {
            words[29].store(text.as_ptr().addr() as u64, Ordering::SeqCst);
            words[30].store(0, Ordering::SeqCst);
            words[32].store(text.len() as u64, Ordering::SeqCst);
        }
        words[31].store(text.len() as u64, Ordering::SeqCst);
        BLOCKS[kind].0[13].store(std::ptr::from_ref(&TYPES[kind]).addr() as u64, Ordering::SeqCst);
    }

    fn put(offset: usize, bytes: &[u8]) {
        for (n, byte) in bytes.iter().enumerate() {
            CHUNK.0[offset + n].store(*byte, Ordering::SeqCst);
        }
    }

    /// Puts the function at `function` in `slot` of the vtable of every stand-in block type.
    pub fn set_type_slot(slot: usize, function: usize) {
        TYPE_VTABLE[slot].store(function, Ordering::SeqCst);
    }

    /// The address of the stand-in `BlockType` of `kind`.
    pub fn block_type(kind: ChunkKind) -> usize {
        std::ptr::from_ref(&TYPES[kind as usize]).addr()
    }

    /// The address of the stand-in `Block` of `kind`.
    pub fn block(kind: ChunkKind) -> usize {
        std::ptr::from_ref(&BLOCKS[kind as usize]).addr()
    }

    /// Sets `mDimension` of the stand-in chunk.
    pub fn set_dimension(dimension: usize) {
        Self::put(0x58, &dimension.to_le_bytes());
    }

    /// Sets the x and z of `mMin`, the world position of the lowest corner of the stand-in chunk.
    pub fn set_corner(x: i32, z: i32) {
        Self::put(0x60, &x.to_le_bytes());
        Self::put(0x68, &z.to_le_bytes());
    }

    /// The address of the stand-in chunk at chunk position (0, 0).
    pub fn chunk() -> usize {
        std::ptr::from_ref(&CHUNK).addr()
    }

    /// Sets `mLoadState`; 11 is loaded.
    pub fn set_load_state(value: u8) {
        Self::put(0xF0, &[value]);
    }

    /// Makes the sub-chunk `index`, counted from the bottom, a client-request placeholder or not.
    pub fn set_placeholder(index: usize, placeholder: bool) {
        PLACEHOLDERS[index].store(placeholder, Ordering::SeqCst);
    }

    /// Sets the height of the column (`x`, `z`) in the heightmap and the render heightmap, as the block y of the top.
    pub fn set_height(x: usize, z: usize, y: i32) {
        let raw = i16::try_from(y - Self::MIN_Y).unwrap().to_le_bytes();
        Self::put(usize::try_from(HEIGHTMAP_AT.load(Ordering::SeqCst)).unwrap() + (z * 16 + x) * 2, &raw);
        Self::put(usize::try_from(RENDER_AT.load(Ordering::SeqCst)).unwrap() + (z * 16 + x) * 2, &raw);
    }

    /// Puts a block of `kind` at the world block y `y` of the column (`x`, `z`).
    pub fn set_block(x: u8, z: u8, y: i32, kind: ChunkKind) {
        let relative = i16::try_from(y - Self::MIN_Y).unwrap();
        PLACED.lock().unwrap().get_or_insert_with(HashMap::new).insert((x, z, relative), kind as usize);
    }

    /// Fills the column (`x`, `z`) with `kind` from world block y `from` to `to`, both included.
    pub fn fill(x: u8, z: u8, from: i32, to: i32, kind: ChunkKind) {
        for y in from..=to {
            Self::set_block(x, z, y, kind);
        }
    }

    /// The block positions `getBlock` was asked for, in order, as column and y relative to the lowest block y.
    pub fn asked() -> Vec<(u8, u8, i16)> {
        ASKED.lock().unwrap().clone()
    }

    /// `LevelChunk* BlockSource::getChunk(int, int) const`: only chunk (0, 0) exists.
    pub extern "C" fn get_chunk(_this: *mut c_void, x: i32, z: i32) -> *mut c_void {
        if (x, z) == (0, 0) { std::ptr::from_ref(&CHUNK).cast_mut().cast() } else { std::ptr::null_mut() }
    }

    // `Block const& LevelChunk::getBlock(ChunkBlockPos const&) const`: air where nothing was placed.
    extern "C" fn get_block(_this: *mut c_void, pos: *const [u8; 4]) -> *mut c_void {
        // SAFETY: the adapter passes a live `ChunkBlockPos` of four bytes: x, z, then y as an i16.
        let bytes = unsafe { pos.read() };
        let key = (bytes[0], bytes[1], i16::from_le_bytes([bytes[2], bytes[3]]));
        ASKED.lock().unwrap().push(key);
        let kind = PLACED.lock().unwrap().as_ref().and_then(|placed| placed.get(&key).copied()).unwrap_or(0);
        std::ptr::from_ref(&BLOCKS[kind]).cast_mut().cast()
    }

    // `bool SubChunk::isPlaceHolderSubChunk() const`: the sub-chunk is told by its place in the vector.
    extern "C" fn is_placeholder(this: *mut c_void) -> u8 {
        let index = (this.addr() - std::ptr::from_ref(&SUB_CHUNKS).addr()) / 0x68;
        u8::from(PLACEHOLDERS[index].load(Ordering::SeqCst))
    }
}

// Where the heightmaps of the layout `setup` chose are.
static HEIGHTMAP_AT: AtomicU64 = AtomicU64::new(0);
static RENDER_AT: AtomicU64 = AtomicU64::new(0);
