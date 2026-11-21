use std::ffi::c_void;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use amml_data::{EntryKind, Resolved};

use super::{ActorGame, ChunkGame, ChunkKind, FakeVtable, Harness};

/// The shape of the map colour calls the stand-in game has.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColorShape {
    /// 26.63: the map colour component and its biome policy.
    Component,
    /// 70.x: `Block::getMapColor`.
    Block,
}

/// A `BlockType::getMapColor` call: the type, the block, the block source and the world position.
pub type TypeCall = (usize, usize, usize, [i32; 3]);

/// A sampler call the stand-in game saw: the entry, the biome id and the world position.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Sampled {
    pub entry: &'static str,
    pub biome: u16,
    pub pos: [i32; 3],
}

// One stand-in `Biome`: the id (two bytes) at 10h, then the `HashedString` at 20h of the 30h layout: the hash word,
// the short text, its size and its capacity.
#[repr(C, align(16))]
struct BiomeWords([AtomicU64; 10]);

// A stand-in `BlockMapColorComponent`: the colour, then the tint method byte at 10h.
#[repr(C)]
struct ComponentObject {
    color: [f32; 4],
    tint: u8,
}

// A stand-in `MapPolicy`: its vtable, with `get` in slot 1.
#[repr(C)]
struct PolicyObject {
    vtable: &'static FakeVtable<2>,
}

// SAFETY: the statics hold plain data and a static table of code addresses.
unsafe impl Sync for PolicyObject {}

const PLAINS: usize = 0;
const SWAMP: usize = 1;
static BIOMES: [BiomeWords; 2] = [const { BiomeWords([const { AtomicU64::new(0) }; 10]) }; 2];
static COMPONENT: ComponentObject = ComponentObject { color: [0.5, 0.5, 0.5, 1.0], tint: 3 };
static POLICY_VTABLE: FakeVtable<2> = FakeVtable([ColorGame::unused as *const (), ColorGame::policy_get as *const ()]);
static POLICY: PolicyObject = PolicyObject { vtable: &POLICY_VTABLE };
static SAMPLED: Mutex<Vec<Sampled>> = Mutex::new(Vec::new());
// The block source and position each `getMapColor` call saw.
static COLOR_CALLS: Mutex<Vec<(usize, [i32; 3])>> = Mutex::new(Vec::new());
static TINTS: Mutex<Vec<u8>> = Mutex::new(Vec::new());
// The `BlockType::getMapColor` calls: the type, the block, the block source and the position.
static TYPE_CALLS: Mutex<Vec<TypeCall>> = Mutex::new(Vec::new());
// Whether `getBiome` gives no biome.
static NO_BIOME: AtomicBool = AtomicBool::new(false);

/// Stand-ins for the map colour and biome calls of the game, over the stand-in chunk of `ChunkGame`: `getBiome` gives
/// plains (id 1) for the columns with z below 8 and swamp (id 6) for the others; the samplers give colours that name
/// the biome; the map colour of stone is 80h, 40h, FFh (the block shape) or the grey component tinted (the component
/// shape).
pub struct ColorGame;

impl ColorGame {
    pub const PLAINS_ID: u16 = 1;
    pub const SWAMP_ID: u16 = 6;
    /// The chunk corner the stand-in chunk reports as `mMin`: the world x and z of its column (0, 0).
    pub const MIN_X: i32 = 32;
    pub const MIN_Z: i32 = -16;
    /// The slot of `BlockType::getMapColor` in the stand-in block type vtable.
    pub const TYPE_COLOR_SLOT: usize = 0x81;
    const ID_AT: u64 = 0x10;
    const HASH_AT: u64 = 0x20;

    /// The map colour and biome entries of `shape`; `without` names entries to leave out. Calls `setup`.
    pub fn entries(shape: ColorShape, without: &[&str]) -> Vec<(&'static str, Resolved)> {
        Self::setup("plains", "swamp");
        let function = |address: usize| (EntryKind::Function, address as u64 - Harness::BASE);
        let mut more = vec![
            ("LevelChunk.getBiome", function(Self::get_biome as *const () as usize)),
            ("BiomeColorSampling.getWaterColor", function(Self::water_color as *const () as usize)),
            ("BiomeColorSampling.getMapGrassColor", function(Self::grass_color as *const () as usize)),
            ("Level.getBiomeRegistry", (EntryKind::VSlot, 5)),
            ("BiomeRegistry.lookupById", function(Self::unused as *const () as usize)),
            ("ClientInstance.getLevel", (EntryKind::VSlot, 6)),
            ("Biome.mId", (EntryKind::Field, Self::ID_AT)),
            ("Biome.mHash", (EntryKind::Field, Self::HASH_AT)),
        ];
        match shape {
            ColorShape::Block => more.push(("Block.getMapColor", function(Self::get_map_color as *const () as usize))),
            ColorShape::Component => more.extend([
                ("Block.tryGetComponent.BlockMapColorComponent", function(Self::try_component as *const () as usize)),
                ("BiomeColorSampling.getMapPolicy", function(Self::get_policy as *const () as usize)),
                ("MapPolicy.get", (EntryKind::VSlot, 1)),
                ("BlockType.getMapColor", (EntryKind::VSlot, Self::TYPE_COLOR_SLOT as u64)),
            ]),
        }
        more.into_iter()
            .filter(|(name, _)| !without.contains(name))
            .map(|(name, (kind, value))| (name, Resolved { kind, value, recipes_agreeing: 2 }))
            .collect()
    }

    /// Makes the two stand-in biomes with the names `plains` and `swamp` (each shorter than 16 bytes) and clears the
    /// calls the stand-ins saw. Also puts the dimension and the corner of the chunk into the stand-in chunk.
    pub fn setup(plains: &str, swamp: &str) {
        Self::name_biome(PLAINS, Self::PLAINS_ID, plains);
        Self::name_biome(SWAMP, Self::SWAMP_ID, swamp);
        ChunkGame::set_dimension(ActorGame::dimension());
        ChunkGame::set_corner(Self::MIN_X, Self::MIN_Z);
        SAMPLED.lock().unwrap().clear();
        COLOR_CALLS.lock().unwrap().clear();
        TINTS.lock().unwrap().clear();
        TYPE_CALLS.lock().unwrap().clear();
        NO_BIOME.store(false, Ordering::SeqCst);
        ChunkGame::set_type_slot(Self::TYPE_COLOR_SLOT, Self::type_map_color as *const () as usize);
    }

    /// Gives the stand-in biome of `swamp` (the one of the columns with z 8 and above) the name `name` and keeps its id.
    pub fn rename_swamp(name: &str) {
        Self::name_biome(SWAMP, Self::SWAMP_ID, name);
    }

    fn name_biome(index: usize, id: u16, name: &str) {
        let words = &BIOMES[index].0;
        let mut inline = [0u8; 16];
        inline[..name.len()].copy_from_slice(name.as_bytes());
        words[2].store(u64::from(id), Ordering::SeqCst);
        // The hash word at 20h, the text at 28h, its size at 38h and its capacity at 40h.
        words[4].store(0, Ordering::SeqCst);
        words[5].store(u64::from_le_bytes(inline[..8].try_into().unwrap()), Ordering::SeqCst);
        words[6].store(u64::from_le_bytes(inline[8..].try_into().unwrap()), Ordering::SeqCst);
        words[7].store(name.len() as u64, Ordering::SeqCst);
        words[8].store(15, Ordering::SeqCst);
    }

    /// The sampler calls the stand-ins saw, in order.
    pub fn sampled() -> Vec<Sampled> {
        SAMPLED.lock().unwrap().clone()
    }

    /// The block source and world position of each `getMapColor` call.
    pub fn color_calls() -> Vec<(usize, [i32; 3])> {
        COLOR_CALLS.lock().unwrap().clone()
    }

    /// The `BlockType::getMapColor` calls: the type, the block, the block source and the world position.
    pub fn type_color_calls() -> Vec<TypeCall> {
        TYPE_CALLS.lock().unwrap().clone()
    }

    /// Makes `getBiome` give no biome (null), or the stand-in biomes again.
    pub fn set_no_biome(none: bool) {
        NO_BIOME.store(none, Ordering::SeqCst);
    }

    /// The tint methods `getMapPolicy` was asked for.
    pub fn tints() -> Vec<u8> {
        TINTS.lock().unwrap().clone()
    }

    extern "C" fn unused(_this: *mut c_void) -> u8 {
        0
    }

    // `Biome const& LevelChunk::getBiome(ChunkBlockPos const&) const`.
    extern "C" fn get_biome(_chunk: *mut c_void, pos: *const [u8; 4]) -> *mut c_void {
        if NO_BIOME.load(Ordering::SeqCst) {
            return std::ptr::null_mut();
        }
        // SAFETY: the adapter passes a live `ChunkBlockPos` of four bytes: x, z, then y as an i16.
        let bytes = unsafe { pos.read() };
        let index = if bytes[1] < 8 { PLAINS } else { SWAMP };
        std::ptr::from_ref(&BIOMES[index]).cast_mut().cast()
    }

    fn biome_id(biome: *mut c_void) -> u16 {
        // SAFETY: the adapter passes the address of a stand-in biome.
        unsafe { biome.cast::<u8>().add(0x10).cast::<u16>().read() }
    }

    // `int BiomeColorSampling::getWaterColor(Biome const&, BlockPos const&)`: 0xFF102030 plus the id.
    extern "C" fn water_color(biome: *mut c_void, pos: *const [i32; 3]) -> u32 {
        let id = Self::biome_id(biome);
        // SAFETY: the adapter passes a live `BlockPos` of three ints.
        let pos = unsafe { pos.read() };
        SAMPLED.lock().unwrap().push(Sampled { entry: "water", biome: id, pos });
        0xFF10_2030 + u32::from(id)
    }

    // `int BiomeColorSampling::getMapGrassColor(Biome const&, BlockPos const&)`: 0x00405060 plus the id.
    extern "C" fn grass_color(biome: *mut c_void, pos: *const [i32; 3]) -> u32 {
        let id = Self::biome_id(biome);
        // SAFETY: the adapter passes a live `BlockPos` of three ints.
        let pos = unsafe { pos.read() };
        SAMPLED.lock().unwrap().push(Sampled { entry: "grass", biome: id, pos });
        0x0040_5060 + u32::from(id)
    }

    // `mce::Color Block::getMapColor(BlockSource const&, BlockPos const&) const`.
    extern "C" fn get_map_color(
        block: *mut c_void,
        out: *mut [f32; 4],
        source: *mut c_void,
        pos: *const [i32; 3],
    ) -> *mut [f32; 4] {
        let color = if block.addr() == ChunkGame::block(ChunkKind::Stone) {
            [128.0 / 255.0, 64.0 / 255.0, 1.0, 1.0]
        } else {
            [0.0, 0.0, 0.0, 1.0]
        };
        // SAFETY: the adapter passes a live `BlockPos` of three ints and a writable colour.
        let pos = unsafe {
            out.write(color);
            pos.read()
        };
        COLOR_CALLS.lock().unwrap().push((source.addr(), pos));
        out
    }

    // `mce::Color BlockType::getMapColor(BlockSource const&, BlockPos const&, Block const&) const`: the block is the fifth
    // argument, on the stack. Gives 0.25, 0.5, 0.75.
    extern "C" fn type_map_color(
        block_type: *mut c_void,
        out: *mut [f32; 4],
        source: *mut c_void,
        pos: *const [i32; 3],
        block: *mut c_void,
    ) -> *mut [f32; 4] {
        // SAFETY: the adapter passes a live `BlockPos` of three ints and a writable colour.
        unsafe {
            out.write([0.25, 0.5, 0.75, 1.0]);
            TYPE_CALLS.lock().unwrap().push((block_type.addr(), block.addr(), source.addr(), pos.read()));
        }
        out
    }

    // `BlockMapColorComponent const* Block::tryGetComponent<BlockMapColorComponent>() const`: stone has one.
    extern "C" fn try_component(block: *mut c_void) -> *const c_void {
        if block.addr() == ChunkGame::block(ChunkKind::Stone) {
            std::ptr::from_ref(&COMPONENT).cast()
        } else {
            std::ptr::null()
        }
    }

    // `MapPolicy const& BiomeColorSampling::getMapPolicy(TintMethod)`.
    extern "C" fn get_policy(tint: u8) -> *mut c_void {
        TINTS.lock().unwrap().push(tint);
        std::ptr::from_ref(&POLICY).cast_mut().cast()
    }

    // `mce::Color MapPolicy::get(BlockSource const&, BlockPos const&) const`: the tint 0.5, 1, 0.
    extern "C" fn policy_get(
        _policy: *mut c_void,
        out: *mut [f32; 4],
        source: *mut c_void,
        pos: *const [i32; 3],
    ) -> *mut [f32; 4] {
        // SAFETY: the adapter passes a live `BlockPos` of three ints and a writable colour.
        unsafe {
            COLOR_CALLS.lock().unwrap().push((source.addr(), pos.read()));
            out.write([0.5, 1.0, 0.0, 1.0]);
        }
        out
    }
}
