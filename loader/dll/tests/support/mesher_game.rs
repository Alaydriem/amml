use std::collections::HashSet;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU8, Ordering};

use amml::HashedStringLayout30;
use amml_data::{EntryKind, Resolved};

use super::{BlockReadGame, ChunkGame, ChunkKind, FakeVtable, Harness, HeapProbe, RenderGame};

// The game calls of the block mesher, one line each, in order.
static LOG: Mutex<Vec<String>> = Mutex::new(Vec::new());
// The stand-in `minecraft:info_update` block that `setRegion` fills the window with, which the cache getters read from
// the world; and the block type of the stand-in air, which the registry lookup gives.
static PLACEHOLDER: [u64; 16] = [0; 16];
static AIR_TYPE: [u64; 16] = [0; 16];
// The registry the loader passes: `BlockTypeRegistry.instance` + 18h.
static REGISTRY: [u64; 8] = [0; 8];
// A stand-in tessellation policy: its vtable, with the four-argument `get` in slot 2 (the survey, row M8).
#[repr(C)]
struct PolicyObject {
    vtable: &'static FakeVtable<3>,
}

// SAFETY: the static holds a reference to a static table of code addresses.
unsafe impl Sync for PolicyObject {}

static POLICY_VTABLE: FakeVtable<3> =
    FakeVtable([MesherGame::unused as *const (), MesherGame::unused as *const (), MesherGame::policy_get as *const ()]);
static POLICY: PolicyObject = PolicyObject { vtable: &POLICY_VTABLE };
// The stand-in biome every column has.
static BIOME: [u64; 4] = [0; 4];
// The method `getTessellationPolicy` was last asked for.
static POLICY_METHOD: AtomicU8 = AtomicU8::new(0);
// The stand-in `BlockGraphics` of each block kind: the `BlockShape` at 10h. The flower is a crossed plant (shape 1).
static GRAPHICS: [[u32; 8]; 7] = {
    let mut graphics = [[0u32; 8]; 7];
    graphics[ChunkKind::Flower as usize][4] = 1;
    graphics
};

/// Stand-ins with the game signatures for the block mesher: `BlockTessellator` (constructor, destructor, `setRegion`,
/// `tessellateInWorld`), `BlockGraphics::getForBlock`, `Block::getRenderLayer` and the `getExtraRenderLayers` slot of the
/// stand-in block types, over the world of `BlockReadGame` and the frame of `RenderGame`. `setRegion` fills the cache
/// window with air, as both gate builds do. `tessellateInWorld` records its arguments and what the window holds at the
/// block, then adds the top face of the block (one quad, white) to the tessellator.
pub struct MesherGame;

impl MesherGame {
    /// The offset of `mBlockCache` in `BlockTessellator`.
    pub const CACHE_AT: usize = 0x6B0;
    // `mBlocks` in `BlockTessellatorCache`.
    const BLOCKS_AT: usize = 0x3EA0;
    // `getExtraRenderLayers` in the stand-in block type vtable.
    const EXTRA_LAYERS_SLOT: usize = 0x7C;
    /// The extra layer bit of water: water is tessellated on its main layer 0 and on layer 3.
    pub const WATER_EXTRA: u32 = 1 << 3;
    /// The offset of `mTintMethod` in the stand-in block type: the word after the name `HashedString`.
    const TINT_AT: usize = 0x108;
    /// The water colour the stand-in biome gives, as the game packs it (alpha, red, green, blue).
    pub const WATER_COLOR: u32 = 0xFF20_4060;
    /// The colour the stand-in tessellation policy gives, red, green, blue, alpha.
    pub const POLICY_COLOR: [f32; 4] = [0.5, 1.0, 0.25, 1.0];

    /// The entries of the block reads, the render entries of `RenderGame` that are not among them, and the mesher entries;
    /// `without` names entries to leave out. Clears the recorded calls.
    pub fn entries(without: &[&str]) -> Vec<(&'static str, Resolved)> {
        let mut entries = BlockReadGame::entries(without);
        ChunkGame::set_type_slot(Self::EXTRA_LAYERS_SLOT, Self::extra_layers as *const () as usize);
        let present: HashSet<&str> = entries.iter().map(|(name, _)| *name).collect();
        let render: Vec<(&'static str, Resolved)> =
            RenderGame::entries(without).into_iter().filter(|(name, _)| !present.contains(name)).collect();
        entries.extend(render);
        // The option registry of `ClientGame` has five slots: the graphics mode read uses its first bool getter
        // (`getHideHud`, false), so the world renders in the classic mode.
        for (name, resolved) in &mut entries {
            if *name == "IOptionRegistry.isGraphicsModeDeferred" {
                resolved.value = 0;
            }
        }
        LOG.lock().unwrap().clear();
        let function = |address: usize| (EntryKind::Function, address as u64 - Harness::BASE);
        let more = [
            ("BlockTessellator.ctor", function(Self::ctor as *const () as usize)),
            ("BlockTessellator.dtor", function(Self::dtor as *const () as usize)),
            ("BlockTessellator.setRegion", function(Self::set_region as *const () as usize)),
            ("BlockTessellator.tessellateInWorld", function(Self::tessellate as *const () as usize)),
            ("BlockTessellator.mBlockCache", (EntryKind::Field, Self::CACHE_AT as u64)),
            ("BlockGraphics.getForBlock", function(Self::graphics as *const () as usize)),
            ("BlockGraphics.mBlockShape", (EntryKind::Field, 0x10)),
            ("Block.getRenderLayer", function(Self::render_layer as *const () as usize)),
            ("BlockType.getExtraRenderLayers", (EntryKind::VSlot, Self::EXTRA_LAYERS_SLOT as u64)),
            ("BlockTypeRegistry.instance", (EntryKind::Global, REGISTRY.as_ptr().addr() as u64 - 0x18 - Harness::BASE)),
            ("BlockTypeRegistry.lookupByName", function(Self::lookup_by_name as *const () as usize)),
            ("BlockType.mTintMethod", (EntryKind::Field, Self::TINT_AT as u64)),
            ("BiomeColorSampling.getTessellationPolicy", function(Self::policy_of as *const () as usize)),
            ("TessellationPolicy.get", (EntryKind::VSlot, 2)),
            ("MeshData.clear", function(Self::data_clear as *const () as usize)),
            ("MeshData.dtor", function(Self::data_dtor as *const () as usize)),
            ("MeshData.moveAssign", function(Self::data_move as *const () as usize)),
            ("LevelChunk.getBiome", function(Self::get_biome as *const () as usize)),
            ("BiomeColorSampling.getWaterColor", function(Self::water_color as *const () as usize)),
        ];
        for (name, (kind, value)) in more {
            if !without.contains(&name) {
                entries.push((name, Resolved { kind, value, recipes_agreeing: 2 }));
            }
        }
        entries
    }

    /// The mesher calls so far.
    pub fn log() -> Vec<String> {
        LOG.lock().unwrap().clone()
    }

    fn record(line: String) {
        LOG.lock().unwrap().push(line);
    }

    /// Gives the stand-in block type of `kind` the tint method `method` (`BlockType.mTintMethod`).
    pub fn set_tint(kind: ChunkKind, method: u8) {
        // SAFETY: the stand-in block type is 110h bytes; the method byte lies at `TINT_AT`.
        unsafe { ((ChunkGame::block_type(kind) + Self::TINT_AT) as *mut u8).write(method) };
    }

    extern "C" fn unused() -> usize {
        0
    }

    // `TessellationPolicy const& BiomeColorSampling::getTessellationPolicy(TintMethod)`: the one stand-in policy.
    extern "C" fn policy_of(method: u8) -> usize {
        POLICY_METHOD.store(method, Ordering::SeqCst);
        std::ptr::from_ref(&POLICY).addr()
    }

    // `mce::Color TessellationPolicy::get(Block const&, BlockSource&, BlockPos const&, BiomeTintCache const*) const`: the
    // colour comes back through the hidden pointer; the position and the tint cache are on the stack.
    extern "C" fn policy_get(
        _policy: usize,
        out: *mut [f32; 4],
        block: usize,
        region: usize,
        pos: *const [i32; 3],
        cache: usize,
    ) -> *mut [f32; 4] {
        // SAFETY: the loader passes a live position and a writable colour.
        let pos = unsafe { pos.read() };
        Self::record(format!(
            "policy get method={} block={} pos={},{},{} region={} cache={}",
            POLICY_METHOD.load(Ordering::SeqCst),
            Self::kind(block),
            pos[0],
            pos[1],
            pos[2],
            region == super::ActorGame::block_source(),
            if cache == 0 { "null" } else { "set" },
        ));
        // SAFETY: `out` is the loader's 16-byte colour.
        unsafe { out.write(Self::POLICY_COLOR) };
        out
    }

    // `Biome const& LevelChunk::getBiome(ChunkBlockPos const&) const`: the one stand-in biome; the column and the y
    // relative to the lowest block y of the chunk are recorded.
    extern "C" fn get_biome(_chunk: usize, pos: *const [u8; 4]) -> usize {
        // SAFETY: the loader passes a live `ChunkBlockPos` of four bytes: x, z, then y as an i16.
        let bytes = unsafe { pos.read() };
        Self::record(format!("biome column={},{} y={}", bytes[0], bytes[1], i16::from_le_bytes([bytes[2], bytes[3]])));
        std::ptr::from_ref(&BIOME).addr()
    }

    // `int BiomeColorSampling::getWaterColor(Biome const&, BlockPos const&)`: the packed colour in eax.
    extern "C" fn water_color(biome: usize, pos: *const [i32; 3]) -> u32 {
        assert_eq!(biome, std::ptr::from_ref(&BIOME).addr());
        // SAFETY: the loader passes a live position.
        let pos = unsafe { pos.read() };
        Self::record(format!("water color pos={},{},{}", pos[0], pos[1], pos[2]));
        Self::WATER_COLOR
    }

    // `mce::MeshData::clear()`: the mode byte and the bounds; the stand-in vectors stay.
    extern "C" fn data_clear(this: usize) {
        Self::record("mesh data clear".to_string());
        // SAFETY: `this` is a MeshData-sized buffer the loader made.
        unsafe { (this as *mut u8).write(0) };
    }

    // The vertices a `MeshData` holds: its positions at 8, first and last.
    //
    // Safety: `data` is a MeshData-sized buffer the loader made.
    unsafe fn data_vertices(data: usize) -> usize {
        // SAFETY: the position vector words lie at 8 and 10h.
        let (first, last) = unsafe { (((data + 8) as *const usize).read(), ((data + 0x10) as *const usize).read()) };
        (last - first) / 12
    }

    // `mce::MeshData::~MeshData()`: records the vertices it held.
    extern "C" fn data_dtor(this: usize) {
        // SAFETY: `this` is a MeshData-sized buffer the loader made.
        Self::record(format!("mesh data dtor vertices={}", unsafe { Self::data_vertices(this) }));
    }

    // `mce::MeshData& mce::MeshData::operator=(mce::MeshData&&)`: the whole object moves; the source is left empty.
    extern "C" fn data_move(this: usize, source: usize) -> usize {
        // SAFETY: both are MeshData-sized buffers; they do not overlap.
        unsafe {
            Self::record(format!("mesh data move vertices={}", Self::data_vertices(source)));
            std::ptr::copy_nonoverlapping(source as *const u8, this as *mut u8, 0x160);
            std::ptr::write_bytes(source as *mut u8, 0, 0x160);
        }
        this
    }

    // The name of the stand-in block at `address`.
    fn kind(address: usize) -> &'static str {
        let kinds = [
            (ChunkKind::Stone, "stone"),
            (ChunkKind::Water, "water"),
            (ChunkKind::Dirt, "dirt"),
            (ChunkKind::Flower, "flower"),
            (ChunkKind::Slab, "slab"),
        ];
        if address == ChunkGame::air() {
            return "air";
        }
        if address == PLACEHOLDER.as_ptr().addr() {
            return "info_update";
        }
        kinds.iter().find(|(kind, _)| ChunkGame::block(*kind) == address).map_or("other", |(_, name)| name)
    }

    fn kind_index(block: usize) -> Option<usize> {
        let kinds = [ChunkKind::Stone, ChunkKind::Water, ChunkKind::Dirt, ChunkKind::Flower, ChunkKind::Slab];
        kinds.iter().find(|kind| ChunkGame::block(**kind) == block).map(|kind| *kind as usize)
    }

    // `BlockTessellator::BlockTessellator(BlockSource&)`: `this` back in rax; the region at 8.
    extern "C" fn ctor(this: usize, region: usize) -> usize {
        Self::record(format!("ctor source={}", region == super::ActorGame::block_source()));
        // SAFETY: `this` is a block of sizeof(BlockTessellator) from the game heap.
        unsafe { ((this + 8) as *mut usize).write_unaligned(region) };
        this
    }

    extern "C" fn dtor(_this: usize) {
        Self::record("dtor".to_string());
    }

    // `void BlockTessellator::setRegion(BlockSource&)`: the region at 8 and in the cache at 18h, the window corner at
    // (-2, -2, -2), every block and extra block of the window the `info_update` placeholder (both gate builds).
    extern "C" fn set_region(this: usize, region: usize) {
        Self::record(format!("set region source={}", region == super::ActorGame::block_source()));
        let cache = this + Self::CACHE_AT;
        // SAFETY: `this` is a block of sizeof(BlockTessellator); the cache lies inside it.
        unsafe {
            ((this + 8) as *mut usize).write_unaligned(region);
            ((cache + 0x18) as *mut usize).write_unaligned(region);
            ((cache + 8) as *mut [i32; 3]).write_unaligned([-2; 3]);
            for index in 0..16_000 {
                ((cache + Self::BLOCKS_AT + index * 8) as *mut usize).write_unaligned(PLACEHOLDER.as_ptr().addr());
            }
        }
    }

    // `bool BlockTessellator::tessellateInWorld(Tessellator&, Block const&, BlockPos const&, bool)`: records the block,
    // the position, the layer at 80h, the flag, the flags at 1 and E8h, the block the cache gives at the position and in
    // the cell above, its extra block at the position and its air at +0, then adds the top face of the block. As the
    // game's cache getters do, a null cell or the placeholder is read from the world (`BlockSource::getBlock` of the
    // region at 18h).
    extern "C" fn tessellate(this: usize, tessellator: usize, block: usize, pos: *const [i32; 3], cached: u8) -> u8 {
        // SAFETY: the loader passes a live position and its block tessellator.
        let (pos, layer, normals, occlusion, corner) = unsafe {
            (
                pos.read(),
                ((this + 0x80) as *const i32).read_unaligned(),
                ((this + 1) as *const u8).read(),
                ((this + 0xE8) as *const u8).read(),
                ((this + Self::CACHE_AT + 8) as *const [i32; 3]).read_unaligned(),
            )
        };
        let cache = this + Self::CACHE_AT;
        let read = |at: usize| {
            // SAFETY: every address read is inside the stand-in's block of sizeof(BlockTessellator).
            unsafe { (at as *const usize).read_unaligned() }
        };
        let cell = |layer: usize, at: [i32; 3]| {
            let [x, y, z] = [at[0] - corner[0], at[1] - corner[1], at[2] - corner[2]];
            let index = (x * 400 + y * 20 + z) as usize;
            let block = read(cache + Self::BLOCKS_AT + layer * 8000 * 8 + index * 8);
            if block == 0 || block == PLACEHOLDER.as_ptr().addr() {
                let world = ChunkGame::block_source_get_block(read(cache + 0x18), &raw const at).addr();
                return format!("world:{}", Self::kind(world));
            }
            Self::kind(block).to_string()
        };
        Self::record(format!(
            "tessellate {} pos={},{},{} layer={layer} cached={cached} normals={normals} ao={occlusion} window={} above={} extra={} air0={}",
            Self::kind(block),
            pos[0],
            pos[1],
            pos[2],
            cell(0, pos),
            cell(0, [pos[0], pos[1] + 1, pos[2]]),
            cell(1, pos),
            Self::kind(read(cache)),
        ));
        let [x, y, z] = pos.map(|axis| axis as f32);
        let top = [[x, y + 1.0, z], [x + 1.0, y + 1.0, z], [x + 1.0, y + 1.0, z + 1.0], [x, y + 1.0, z + 1.0]];
        // SAFETY: the loader's tessellator is the stand-in one, with an open list.
        unsafe {
            Self::push(tessellator + 0x10, &top);
            Self::push(tessellator + 0x70, &[0xFFFF_FFFFu32; 4]);
            let count = (tessellator + 0x298) as *mut u32;
            count.write(count.read() + 4);
        }
        1
    }

    // Appends `items` to the game vector at `vector`, growing it on the stand-in game heap.
    //
    // Safety: `vector` is a vector of `T` whose block came from `HeapProbe`, or null.
    unsafe fn push<T: Copy>(vector: usize, items: &[T]) {
        let size = size_of::<T>();
        // SAFETY: the caller guarantees the three words of the vector.
        unsafe {
            let words = vector as *mut [usize; 3];
            let [first, last, end] = words.read();
            let len = if first == 0 { 0 } else { (last - first) / size };
            let mut target = [first, last, end];
            if first == 0 || last + size_of_val(items) > end {
                let capacity = (len + items.len()).max(2 * len);
                let block = HeapProbe::alloc(capacity * size) as usize;
                if first != 0 {
                    std::ptr::copy_nonoverlapping(first as *const u8, block as *mut u8, len * size);
                    HeapProbe::free(first as *mut u8, end - first);
                }
                target = [block, block + len * size, block + capacity * size];
            }
            for (index, item) in items.iter().enumerate() {
                ((target[1] + index * size) as *mut T).write_unaligned(*item);
            }
            target[1] += size_of_val(items);
            words.write(target);
        }
    }

    // `_lookupByNameImpl(name, data, resolve, logNotFound)`: the stand-in air type and the stand-in air block for
    // `minecraft:air` with the game's hash, nothing for any other name. As in the game, the block (result+8) is
    // filled only for resolve 1 (`Block`); this stand-in has it for data 0 only.
    extern "C" fn lookup_by_name(
        registry: usize,
        out: *mut [usize; 2],
        name: *const HashedStringLayout30,
        data: i32,
        resolve: i32,
        _log: u8,
    ) -> *mut [usize; 2] {
        assert_eq!(registry, REGISTRY.as_ptr().addr());
        // SAFETY: the loader passes a live HashedString of the 30h layout.
        let name = unsafe { &*name };
        let text = String::from_utf8_lossy(name.text.as_bytes()).into_owned();
        Self::record(format!("lookup {text} hash={:#x}", name.hash));
        let found = if text == "minecraft:air" && name.hash == 0xbd58_4baf_0000_3448 {
            [AIR_TYPE.as_ptr().addr(), ChunkGame::air()]
        } else if text == "minecraft:grass_block" {
            // The stand-in dirt is this world's grass block.
            [ChunkGame::block_type(ChunkKind::Dirt), ChunkGame::block(ChunkKind::Dirt)]
        } else {
            [0, 0]
        };
        let found = if resolve == 1 && data == 0 { found } else { [found[0], 0] };
        // SAFETY: `out` is the loader's 16-byte result.
        unsafe { out.write(found) };
        out
    }

    // `static BlockGraphics* BlockGraphics::getForBlock(Block const&)`: the stand-in graphics of the kind; null for air.
    extern "C" fn graphics(block: usize) -> usize {
        Self::kind_index(block).map_or(0, |kind| std::ptr::from_ref(&GRAPHICS[kind]).addr())
    }

    // `BlockRenderLayer Block::getRenderLayer() const`: 0 for every kind.
    extern "C" fn render_layer(_block: usize) -> u8 {
        0
    }

    // `int BlockType::getExtraRenderLayers() const`: layer 3 for water, none for the others.
    extern "C" fn extra_layers(block_type: usize) -> u32 {
        if block_type == ChunkGame::block_type(ChunkKind::Water) { Self::WATER_EXTRA } else { 0 }
    }
}
