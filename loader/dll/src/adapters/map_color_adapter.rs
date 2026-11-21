use std::ffi::c_void;

use amml_abi::AmmlRgba;
use amml_data::EntryId;

use crate::{LoaderState, MapColorComponentLayout, MapColorShape, MceColor, WorldBlockPos};

/// `BlockSource& Dimension::getBlockSourceFromMainChunkSource() const`: rcx the dimension.
type MainSourceFn = unsafe extern "C" fn(*mut c_void) -> *mut c_void;
/// `BlockMapColorComponent const* Block::tryGetComponent<BlockMapColorComponent>() const`: rcx the block.
type ComponentFn = unsafe extern "C" fn(*mut c_void) -> *const MapColorComponentLayout;
/// `MapPolicy const& BiomeColorSampling::getMapPolicy(TintMethod)`: the tint method in cl.
type PolicyFn = unsafe extern "C" fn(u8) -> *mut c_void;
/// `mce::Color MapPolicy::get(BlockSource const&, BlockPos const&) const`: rcx the policy, the colour through rdx, the
/// block source in r8 and the position in r9.
type PolicyGetFn = unsafe extern "C" fn(*mut c_void, *mut MceColor, *mut c_void, *const WorldBlockPos) -> *mut MceColor;
/// `mce::Color BlockType::getMapColor(BlockSource const&, BlockPos const&, Block const&) const`: rcx the type, the
/// colour through rdx, the block source in r8, the position in r9 and the block on the stack.
type TypeColorFn =
    unsafe extern "C" fn(*mut c_void, *mut MceColor, *mut c_void, *const WorldBlockPos, *mut c_void) -> *mut MceColor;
/// `mce::Color Block::getMapColor(BlockSource const&, BlockPos const&) const`: rcx the block, the colour through rdx,
/// the block source in r8 and the position in r9.
type BlockColorFn =
    unsafe extern "C" fn(*mut c_void, *mut MceColor, *mut c_void, *const WorldBlockPos) -> *mut MceColor;
/// `int BiomeColorSampling::getWaterColor(Biome const&, BlockPos const&)` and `getMapGrassColor`: static, the colour
/// as 0xAARRGGBB in eax.
type BiomeSamplerFn = unsafe extern "C" fn(*mut c_void, *const WorldBlockPos) -> u32;

// The calls of the shape the build has.
enum Calls {
    Component { try_component: usize, get_policy: usize, policy_get_slot: usize, type_color_slot: usize },
    Block { block_color: usize },
}

/// The game side of the `world.map_color` area: the map colour of a block, and the water and grass colours of a
/// biome. The calls are for the game thread. 26.63 reads the map colour component of the block and tints it by the
/// biome policy of the component (the colour of the block type when the block has no component); 70.x asks
/// `Block::getMapColor`. Both read the biome through the main chunk source of the dimension of the chunk.
pub struct MapColorAdapter {
    calls: Calls,
    main_source_slot: usize,
    block_type_at: usize,
    water_color: usize,
    grass_color: usize,
}

impl MapColorAdapter {
    /// The adapter for the entries of the build; `None` when no shape of the map colour is complete.
    pub fn new(state: &LoaderState) -> Option<Self> {
        let resolved = |name: &str| state.table().get(&EntryId(name.to_string())).is_some();
        let shape = MapColorShape::pick(&resolved)?;
        let value = |name: &str| state.resolve(name).ok();
        let function = |name: &str| value(name).and_then(|value| usize::try_from(value).ok()).filter(|at| *at != 0);
        let slot = |name: &str| value(name).and_then(|value| usize::try_from(value).ok());
        let calls = match shape {
            MapColorShape::Component => Calls::Component {
                try_component: function("Block.tryGetComponent.BlockMapColorComponent")?,
                get_policy: function("BiomeColorSampling.getMapPolicy")?,
                policy_get_slot: slot("MapPolicy.get")?,
                type_color_slot: slot("BlockType.getMapColor")?,
            },
            MapColorShape::Block => Calls::Block { block_color: function("Block.getMapColor")? },
        };
        Some(Self {
            calls,
            main_source_slot: slot("Dimension.getBlockSourceFromMainChunkSource")?,
            block_type_at: slot("Block.mBlockType")?,
            water_color: function("BiomeColorSampling.getWaterColor")?,
            grass_color: function("BiomeColorSampling.getMapGrassColor")?,
        })
    }

    /// The shape of the build.
    pub fn shape(&self) -> MapColorShape {
        match self.calls {
            Calls::Component { .. } => MapColorShape::Component,
            Calls::Block { .. } => MapColorShape::Block,
        }
    }

    /// The main chunk source of `dimension`; `None` for null.
    ///
    /// # Safety
    /// `dimension` is the live dimension of a live chunk, called on the game thread.
    pub unsafe fn main_source(&self, dimension: usize) -> Option<usize> {
        // SAFETY: `dimension` is live and starts with its vtable.
        let function = unsafe { Self::virtual_fn(dimension, self.main_source_slot) }?;
        // SAFETY: the slot holds `getBlockSourceFromMainChunkSource`, a member that takes only `this`.
        let function = unsafe { std::mem::transmute::<usize, MainSourceFn>(function) };
        // SAFETY: called on the live dimension, on the game thread.
        let source = unsafe { function(std::ptr::with_exposed_provenance_mut(dimension)) }.expose_provenance();
        (source != 0).then_some(source)
    }

    /// The map colour of the live `block` at `pos`, read through `source`; `None` when the game gives no colour.
    ///
    /// # Safety
    /// `block` is a live game `Block` and `source` the live main chunk source of the dimension of its chunk, called on
    /// the game thread.
    pub unsafe fn block_color(&self, block: usize, source: usize, pos: &WorldBlockPos) -> Option<AmmlRgba> {
        let block_ptr = std::ptr::with_exposed_provenance_mut::<c_void>(block);
        let source_ptr = std::ptr::with_exposed_provenance_mut::<c_void>(source);
        let out = match self.calls {
            Calls::Block { block_color } => {
                let mut out = MceColor::default();
                // SAFETY: the entry is `Block::getMapColor`.
                let function = unsafe { std::mem::transmute::<usize, BlockColorFn>(block_color) };
                // SAFETY: every argument lives across the call.
                unsafe { function(block_ptr, &raw mut out, source_ptr, pos) };
                out
            }
            Calls::Component { try_component, get_policy, policy_get_slot, type_color_slot } => {
                // SAFETY: the entry is `Block::tryGetComponent<BlockMapColorComponent>`.
                let component = unsafe { std::mem::transmute::<usize, ComponentFn>(try_component) };
                // SAFETY: called on the live block.
                let component = unsafe { component(block_ptr) };
                if component.is_null() {
                    // SAFETY: the caller's contract.
                    unsafe { self.type_color(block, source_ptr, pos, type_color_slot) }?
                } else {
                    // SAFETY: a non-null component is a live `BlockMapColorComponent`, which the layout mirrors.
                    let component = unsafe { component.read() };
                    // SAFETY: the entry is the static `getMapPolicy`.
                    let policy = unsafe { std::mem::transmute::<usize, PolicyFn>(get_policy) };
                    // SAFETY: `getMapPolicy` takes any tint method.
                    let policy = unsafe { policy(component.tint) };
                    // SAFETY: the game gives a live policy, whose first qword is its vtable.
                    let get = unsafe { Self::virtual_fn(policy.expose_provenance(), policy_get_slot) }?;
                    // SAFETY: the slot holds `MapPolicy::get`.
                    let get = unsafe { std::mem::transmute::<usize, PolicyGetFn>(get) };
                    let mut tint = MceColor::default();
                    // SAFETY: every argument lives across the call.
                    unsafe { get(policy, &raw mut tint, source_ptr, pos) };
                    component.base.tinted(tint)
                }
            }
        };
        Some(out.to_rgba())
    }

    // The colour of the block type of `block`, for a block with no map colour component.
    unsafe fn type_color(
        &self,
        block: usize,
        source: *mut c_void,
        pos: &WorldBlockPos,
        slot: usize,
    ) -> Option<MceColor> {
        // SAFETY: `block` is live and `block_type_at` is the offset of its `mBlockType` pointer.
        let block_type = unsafe {
            std::ptr::with_exposed_provenance::<usize>(block.checked_add(self.block_type_at)?).read_unaligned()
        };
        if block_type == 0 {
            return None;
        }
        // SAFETY: the type is live and starts with its vtable.
        let function = unsafe { Self::virtual_fn(block_type, slot) }?;
        // SAFETY: the slot holds `BlockType::getMapColor`.
        let function = unsafe { std::mem::transmute::<usize, TypeColorFn>(function) };
        let mut out = MceColor::default();
        // SAFETY: every argument lives across the call.
        unsafe {
            function(
                std::ptr::with_exposed_provenance_mut(block_type),
                &raw mut out,
                source,
                pos,
                std::ptr::with_exposed_provenance_mut(block),
            )
        };
        Some(out)
    }

    /// The water colour of the live `biome` at `pos`.
    ///
    /// # Safety
    /// `biome` is a live game `Biome`, called on the game thread.
    pub unsafe fn biome_water(&self, biome: usize, pos: &WorldBlockPos) -> AmmlRgba {
        // SAFETY: as the function's contract; the entry is `getWaterColor`.
        Self::argb(unsafe { Self::sample(self.water_color, biome, pos) })
    }

    /// The map grass colour of the live `biome` at `pos`.
    ///
    /// # Safety
    /// `biome` is a live game `Biome`, called on the game thread.
    pub unsafe fn biome_grass(&self, biome: usize, pos: &WorldBlockPos) -> AmmlRgba {
        // SAFETY: as the function's contract; the entry is `getMapGrassColor`.
        Self::argb(unsafe { Self::sample(self.grass_color, biome, pos) })
    }

    /// A colour of the game as 0xAARRGGBB or 0x00RRGGBB, as bytes with alpha 255.
    pub fn argb(color: u32) -> AmmlRgba {
        let [b, g, r, _] = color.to_le_bytes();
        AmmlRgba { r, g, b, a: 255 }
    }

    unsafe fn sample(function: usize, biome: usize, pos: &WorldBlockPos) -> u32 {
        // SAFETY: the callers pass a static sampler of this signature.
        let function = unsafe { std::mem::transmute::<usize, BiomeSamplerFn>(function) };
        // SAFETY: the biome is live; `pos` lives across the call.
        unsafe { function(std::ptr::with_exposed_provenance_mut(biome), pos) }
    }

    // The function in `slot` of the vtable at the start of `object`.
    unsafe fn virtual_fn(object: usize, slot: usize) -> Option<usize> {
        if object == 0 {
            return None;
        }
        // SAFETY: `object` is a live game object whose first qword is its vtable.
        let vtable = unsafe { std::ptr::with_exposed_provenance::<usize>(object).read() };
        if vtable == 0 {
            return None;
        }
        // SAFETY: the vtable of the object; the entry keeps `slot` inside it.
        let function = unsafe { std::ptr::with_exposed_provenance::<usize>(vtable + slot * size_of::<usize>()).read() };
        (function != 0).then_some(function)
    }
}
