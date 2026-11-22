use amml_runtime::MeshPost;

use crate::{InventoryAdapter, LoaderState, ObjectRead, WorldAdapter};

/// `TessellationPolicy const& BiomeColorSampling::getTessellationPolicy(TintMethod)`: cl the method; the policy in rax
/// (survey section 7, rows M8 and M8b: `cmp cl,9` then a table load, a value of 9 or more reads entry 0).
type PolicyOfFn = unsafe extern "C" fn(u8) -> usize;
/// `mce::Color TessellationPolicy::get(Block const&, BlockSource&, BlockPos const&, BiomeTintCache const*) const`: rcx
/// the policy, rdx the 16-byte colour (the hidden return pointer, given back in rax), r8 the block, r9 the block source,
/// the position at entry rsp+28h and the tint cache at rsp+30h (row M8; `get` of slot 2 reads the same registers on both
/// gate builds, and with a null cache it takes the path that samples the biome directly).
type PolicyGetFn = unsafe extern "C" fn(usize, *mut [f32; 4], usize, usize, *const [i32; 3], usize) -> *mut [f32; 4];
/// `int BiomeColorSampling::getWaterColor(Biome const&, BlockPos const&)`: rcx the biome (rdx is not read); the packed
/// colour, alpha in the top byte, in eax (row M8).
type WaterColorFn = unsafe extern "C" fn(usize, *const [i32; 3]) -> u32;

/// The biome tint of the block mesher: the game's tessellation policy colour for a block whose `BlockType.mTintMethod`
/// asks for one, and the biome water colour for water.
///
/// The game's tessellator leaves a tinted block grey on the cached path, so after each pass of a cell the loader
/// multiplies the grey vertices by the colour of the policy of the block's tint method at the cell's world position (`MeshPost::tint`).
/// Methods 1 to 8 other than 6 call the policy: 1 to 4 sample the biome, 5 reads the chunk's column colour when no tint cache is
/// given, 7 and 8 compute a colour from the block's data (not white; the sampler table of the client maps and the policies of the
/// tessellator are different tables).
/// A block of a tint method the table does not have (9 or more, the game's `cmp cl,9`) has none. The method of water is
/// 6 (entry 6 of the client sampler table is `getWaterColor`, row M8b): it takes the water colour of the biome of the
/// chunk at the cell instead of the policy colour; without the option or
/// without the area it stays grey. A block the game already coloured
/// is left as it is, and the grass block takes its colour on the top face only. The calls run on the game thread, inside
/// a `renderLevel` call.
pub struct MesherTint {
    tint_method_at: usize,
    block_type_at: usize,
    policy_of: PolicyOfFn,
    get_slot: usize,
    water_color: Option<WaterColorFn>,
}

impl MesherTint {
    /// `TintMethod::Water`: the method 6 of the client's sampler table is `getWaterColor` (row M8b).
    const WATER: u8 = 6;
    /// `TintMethod::Size`: the methods the game's policy table holds (row M8b).
    const METHODS: u8 = 9;

    /// The tint calls of this build from the resolved entries; `Err` names the first entry that is missing. The water
    /// colour is optional: without `BiomeColorSampling.getWaterColor` the water stays grey.
    pub fn new(state: &LoaderState) -> Result<Self, &'static str> {
        let value =
            |name: &'static str| state.resolve(name).ok().and_then(|value| usize::try_from(value).ok()).ok_or(name);
        let address =
            |name: &'static str| value(name).and_then(|address| if address == 0 { Err(name) } else { Ok(address) });
        // SAFETY: each address is the derived body of the named function, whose signature is the type it becomes
        // (argument registers read on both gate builds, survey section 7, row M8).
        unsafe {
            Ok(Self {
                tint_method_at: value("BlockType.mTintMethod")?,
                block_type_at: value("Block.mBlockType")?,
                policy_of: std::mem::transmute::<usize, PolicyOfFn>(address(
                    "BiomeColorSampling.getTessellationPolicy",
                )?),
                get_slot: value("TessellationPolicy.get")?,
                water_color: address("BiomeColorSampling.getWaterColor")
                    .ok()
                    .map(|address| std::mem::transmute::<usize, WaterColorFn>(address)),
            })
        }
    }

    /// Whether the water colour of a biome can be read: the entry resolved and the chunk reads give a biome.
    pub fn offers_water(&self) -> bool {
        self.water_color.is_some()
            && WorldAdapter::get().and_then(WorldAdapter::chunks).is_some_and(crate::ChunkAdapter::reads_biomes)
    }

    /// The block type of the live `block`; 0 for none.
    ///
    /// # Safety
    /// `block` is a live game block, on the game thread.
    pub unsafe fn block_type(&self, block: usize) -> usize {
        // SAFETY: a live block holds its block type pointer at `mBlockType`.
        unsafe { ObjectRead::pointer(block, self.block_type_at) }.unwrap_or(0)
    }

    /// Tints the vertices of one pass of the cell `block` at the world position `pos`: `colors` are the packed colours of
    /// the vertices `positions` (the pass's, relative to the grid and pushed into the cell whose centre is `centre`).
    /// `source` is the block source of the local player; `grass_type` the block type of this world's grass block (0 for
    /// none); `water` whether water takes the biome colour (the option is on and the area offered).
    ///
    /// # Safety
    /// `block` is a live game block of the world of the live block source `source`, on the game thread.
    pub unsafe fn apply(
        &self,
        scope: (usize, usize, bool),
        block: usize,
        pos: [i32; 3],
        centre: [f32; 3],
        positions: &[[f32; 3]],
        colors: &mut [u32],
    ) {
        let (source, grass_type, water) = scope;
        // SAFETY: the caller's contract.
        let block_type = unsafe { self.block_type(block) };
        // SAFETY: a live block type holds its tint method byte at the entry's offset (row M8: `mTintMethod` at 17Ch).
        let Some(method) = (unsafe { ObjectRead::value::<u8>(block_type, self.tint_method_at) }) else { return };
        if method == 0 || method >= Self::METHODS {
            return;
        }
        let colour = if method == Self::WATER {
            if !water {
                return;
            }
            // SAFETY: the caller's contract.
            unsafe { self.water_colour(source, pos) }
        } else {
            // SAFETY: the caller's contract.
            unsafe { self.policy_colour(method, block, source, pos) }
        };
        let Some(colour) = colour else { return };
        MeshPost::tint(positions, colors, centre, colour, block_type == grass_type && grass_type != 0);
    }

    // The red, green and blue the game's policy of `method` gives `block` at `pos`, with no tint cache.
    //
    // Safety: a live block of the world of the live block source `source`, on the game thread.
    unsafe fn policy_colour(&self, method: u8, block: usize, source: usize, pos: [i32; 3]) -> Option<[f32; 3]> {
        // SAFETY: a method inside the game's table (checked by the caller).
        let policy = unsafe { (self.policy_of)(method) };
        if policy == 0 {
            return None;
        }
        let get = InventoryAdapter::virtual_fn(policy, self.get_slot)?;
        // SAFETY: the slot of `TessellationPolicy.get` in the vtable of the live policy (row M8).
        let get = unsafe { std::mem::transmute::<usize, PolicyGetFn>(get) };
        let mut colour = [0.0f32; 4];
        // SAFETY: the live policy, a 16-byte colour, the live block and block source and a position that lives for the
        // call; a null tint cache is the game's own "no cache" path.
        unsafe { get(policy, &raw mut colour, block, source, &raw const pos, 0) };
        Some([colour[0], colour[1], colour[2]])
    }

    // The red, green and blue of the water colour of the biome of the chunk at `pos`; `None` without the entry, a chunk or
    // a biome.
    //
    // Safety: the live block source `source` of the local player, on the game thread.
    unsafe fn water_colour(&self, source: usize, pos: [i32; 3]) -> Option<[f32; 3]> {
        let function = self.water_color?;
        let chunks = WorldAdapter::get().and_then(WorldAdapter::chunks)?;
        // SAFETY: the caller's contract.
        let biome = unsafe { chunks.biome_at_world(source, pos) };
        if biome == 0 {
            return None;
        }
        // SAFETY: the live biome the chunk gave and a position that lives for the call.
        let argb = unsafe { function(biome, &raw const pos) };
        let [blue, green, red, _] = argb.to_le_bytes();
        Some([red, green, blue].map(|channel| f32::from(channel) / 255.0))
    }
}
