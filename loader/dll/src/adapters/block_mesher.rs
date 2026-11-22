use std::mem::offset_of;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Mutex, MutexGuard};
use std::time::Instant;

use amml_abi::{AmmlMeshGroup, AmmlStatus};
use amml_msvc::{MsvcHeap, MsvcVec};
use amml_runtime::{CacheWindow, GroupBook, GroupKey, GroupRefusal, MeshPost, WindowCell};

use crate::{
    BlockTessellatorCacheLayout, BlockTessellatorLayout, ClientAdapter, GameVector, InventoryAdapter, LoaderState,
    MeshBuilder, MeshDataBuffer, MeshDataLayout, MesherTint, ObjectRead, RegistryLookup, TessellatorLayout,
};

/// `BlockTessellator::BlockTessellator(BlockSource&)`: rcx this, rdx the region (stored at 8, survey section 7, row M1);
/// `this` back in rax.
type CtorFn = unsafe extern "C" fn(*mut BlockTessellatorLayout, usize) -> *mut BlockTessellatorLayout;
/// `BlockTessellator::~BlockTessellator()`: rcx this only.
type DtorFn = unsafe extern "C" fn(*mut BlockTessellatorLayout);
/// `void BlockTessellator::setRegion(BlockSource&)`: rcx this, rdx the region.
type SetRegionFn = unsafe extern "C" fn(*mut BlockTessellatorLayout, usize);
/// `bool BlockTessellator::tessellateInWorld(Tessellator&, Block const&, BlockPos const&, bool)`: rcx this, rdx the
/// tessellator, r8 the block, r9 the position, the flag at entry rsp+28h (written to F1h); the result in al (row M5).
type TessellateFn =
    unsafe extern "C" fn(*mut BlockTessellatorLayout, *mut TessellatorLayout, usize, *const [i32; 3], u8) -> u8;
/// `static BlockGraphics const* BlockGraphics::getForBlock(Block const&)`: rcx the block; a nullable pointer in rax (row
/// M6).
type GraphicsFn = unsafe extern "C" fn(usize) -> usize;
/// `BlockRenderLayer Block::getRenderLayer() const`: rcx the block; the layer byte in al (row M7).
type RenderLayerFn = unsafe extern "C" fn(usize) -> u8;
/// `int BlockType::getExtraRenderLayers() const`: rcx the block type only; the layer bits in eax (row M7).
type ExtraLayersFn = unsafe extern "C" fn(usize) -> u32;

// One cell to draw: the block, its world position, its render layers and whether it gets back faces.
type Draw = (usize, [i32; 3], Vec<i32>, bool);

// The tessellator of the joined world, its epoch, the air block and the grass block type of that world (0 for none).
type Made = (usize, u64, usize, usize);

// What a grid left in the frame's open list: the list and its vertex count (`None` when nothing was drawn), the grid
// indices of the skipped cells and the number of cells drawn.
type Drawn = (Option<(*mut TessellatorLayout, usize)>, Vec<u32>, usize);

/// The tessellator of the block mesher: one game `BlockTessellator` per world, made on the game heap with the game's
/// constructor at the first build of a world and destroyed at the world leave, on the `renderLevel` thread (the game thread
/// on both gate builds). The game objects are used only on that thread.
///
/// A build points the tessellator at the block source of the local player (`setRegion`, which also sets every block and
/// extra block of the cache window to the `minecraft:info_update` placeholder, which the cache getters read from the world,
/// and clears the cached graphics, both builds), turns the cached calculation on with normals and ambient occlusion off
/// (as the old mesher), writes the real air block (the registry's default state of `minecraft:air`, the block the cache's
/// own getter puts at +0) into +0, every extra block and every window cell the grid leaves empty, fills the rest of the
/// window from the grid (`CacheWindow`) with every light value at 0F0Fh (taken as computed, so the ghost is not dark
/// under a roof), then tessellates each inner cell in each render layer its block needs into the frame's tessellator,
/// applies `MeshPost` to the vertices of each pass and ends the list into a mesh (`MeshBuilder`). The first build, and each later one slower than 2 ms and slower than all before it,
/// logs `mesher build time`.
///
/// A block whose tint method asks for a biome colour is tinted after each pass (`MesherTint`).
///
/// A group (`GroupBook`) is a game `MeshData` buffer (`MeshDataBuffer`) that grids are appended to: an append tessellates
/// the grid as a build does, copies the vertices of the frame's list into the buffer, with the grid moved by the append's
/// offset, and drops the list; the finish moves the buffer into the frame's tessellator as a triangle list with its own
/// index buffer and ends it into one mesh. A group is bound to its world: a world leave, a render device reset and the
/// owner's disable drop it, and its buffer is destroyed on the `renderLevel` thread.
pub struct BlockMesher {
    ctor: CtorFn,
    dtor: DtorFn,
    set_region: SetRegionFn,
    tessellate: TessellateFn,
    graphics: GraphicsFn,
    render_layer: RenderLayerFn,
    extra_layers_slot: usize,
    block_shape_at: usize,
    block_type_at: usize,
    registry: RegistryLookup,
    tint: MesherTint,
    buffers: MeshDataBuffer,
    groups: GroupBook,
    // The tessellator of the joined world with what belongs to that world.
    made: Mutex<Option<Made>>,
    // A tessellator whose world was left off the game thread; destroyed at the next `renderLevel`.
    pending: Mutex<Option<usize>>,
    logged: AtomicBool,
    slowest_us: AtomicU32,
    unmirrored_logged: AtomicBool,
}

impl BlockMesher {
    /// The light value of every window cell: sky and block light 0Fh, at or below FEFFh, so `getLightColor` takes it as
    /// computed on both gate builds (`cmp eax,0FEFFh; jbe`).
    const FULL_LIGHT: u16 = 0x0F0F;
    // The vertices a pass of one cell reserves: a full block's six quads.
    const VERTICES_PER_PASS: usize = 24;
    // A build slower than this is logged when it is the slowest so far.
    const SLOW_US: u32 = 2000;
    const BLOCK_CACHE_ENTRY: &'static str = "BlockTessellator.mBlockCache";
    // The name the cache's own getter looks up for its air block (+0): the global it passes holds the hash
    // BD584BAF00003448h and the text `minecraft:air` on both gate builds.
    const AIR: &'static str = "minecraft:air";
    // The block whose top face takes the biome tint and whose sides do not (the tessellator's texture is the same).
    const GRASS: &'static str = "minecraft:grass_block";
    /// The most vertices one group holds: 65536 quads, which bounds the memory of a group.
    pub const MAX_GROUP_VERTICES: usize = 65_536 * MeshPost::QUAD;

    /// The mesher of this build from the resolved entries; `Err` names the first entry that is missing (a function entry
    /// of value 0 counts as missing), or `BlockTessellator.mBlockCache` when it is not where the mirror has it (`block
    /// tessellator layout mismatch` logs it).
    pub fn new(state: &'static LoaderState) -> Result<Self, &'static str> {
        let value =
            |name: &'static str| state.resolve(name).ok().and_then(|value| usize::try_from(value).ok()).ok_or(name);
        let address =
            |name: &'static str| value(name).and_then(|address| if address == 0 { Err(name) } else { Ok(address) });
        let cache = value(Self::BLOCK_CACHE_ENTRY)?;
        if cache as u64 != BlockTessellatorLayout::BLOCK_CACHE {
            curia::warn!("block tessellator layout mismatch", { entry: Self::BLOCK_CACHE_ENTRY, found: cache });
            return Err(Self::BLOCK_CACHE_ENTRY);
        }
        address("BlockTypeRegistry.instance")?;
        address("BlockTypeRegistry.lookupByName")?;
        value("HashedString.layout")?;
        let registry = RegistryLookup::new(state).ok_or("HashedString.layout")?;
        let tint = MesherTint::new(state)?;
        let buffers = MeshDataBuffer::new(state)?;
        // SAFETY: each address is the derived body of the named function, whose signature is the type it becomes
        // (argument registers read on both gate builds, survey section 7).
        unsafe {
            Ok(Self {
                ctor: std::mem::transmute::<usize, CtorFn>(address("BlockTessellator.ctor")?),
                dtor: std::mem::transmute::<usize, DtorFn>(address("BlockTessellator.dtor")?),
                set_region: std::mem::transmute::<usize, SetRegionFn>(address("BlockTessellator.setRegion")?),
                tessellate: std::mem::transmute::<usize, TessellateFn>(address("BlockTessellator.tessellateInWorld")?),
                graphics: std::mem::transmute::<usize, GraphicsFn>(address("BlockGraphics.getForBlock")?),
                render_layer: std::mem::transmute::<usize, RenderLayerFn>(address("Block.getRenderLayer")?),
                extra_layers_slot: value("BlockType.getExtraRenderLayers")?,
                block_shape_at: value("BlockGraphics.mBlockShape")?,
                block_type_at: value("Block.mBlockType")?,
                registry,
                tint,
                buffers,
                groups: GroupBook::new(),
                made: Mutex::new(None),
                pending: Mutex::new(None),
                logged: AtomicBool::new(false),
                slowest_us: AtomicU32::new(0),
                unmirrored_logged: AtomicBool::new(false),
            })
        }
    }

    /// Builds the grid `cells` of `dims` at the block position `origin` (palette `blocks`, live game blocks of this world)
    /// into the frame's tessellator of `screen` and ends it into a mesh; gives the mesh handle (`None` when nothing was
    /// drawn), its quads and the grid indices of the cells left out because `world_block` gave the same block. `look` is
    /// the alpha of every vertex and whether water takes the biome colour. `FAILED` when the game heap or the game gave
    /// no object.
    ///
    /// # Safety
    /// Called inside the `renderLevel` call whose screen context is `screen`, on the game thread; `source` is the live
    /// block source of the local player of the world of `epoch`; every element of `blocks` is a live game block of that
    /// world; `heap` is the game's allocator.
    #[allow(clippy::too_many_arguments)]
    pub unsafe fn build(
        &self,
        builder: &MeshBuilder,
        screen: usize,
        source: usize,
        epoch: u64,
        grid: ([u32; 3], [i32; 3], &[u16]),
        blocks: &[usize],
        look: (u8, bool),
        world_block: Option<&dyn Fn([i32; 3]) -> usize>,
        heap: &MsvcHeap,
    ) -> Result<(Option<usize>, u32, Vec<u32>), AmmlStatus> {
        let started = Instant::now();
        let origin = grid.1;
        // SAFETY: the caller's contract; the grid stays where it is.
        let (list, skipped, cells) =
            unsafe { self.draw(builder, screen, source, epoch, grid, origin, blocks, look, world_block, heap) }?;
        let Some((list, vertices)) = list else { return Ok((None, 0, skipped)) };
        // SAFETY: the list is open and holds the vertices of the build.
        let handle = unsafe { builder.close(list) }?;
        let quads = u32::try_from(vertices / MeshPost::QUAD).unwrap_or(u32::MAX);
        self.note_time(cells, quads, started);
        Ok((Some(handle), quads, skipped))
    }

    /// Opens a mesh group of the mod `owner`: an empty game `MeshData` buffer. `REFUSED` when the mod holds the most
    /// groups it may.
    pub fn group_begin(&self, owner: usize) -> Result<AmmlMeshGroup, AmmlStatus> {
        let handle = self.buffers.make();
        match self.groups.open(owner, handle) {
            Ok(key) => Ok(AmmlMeshGroup { id: key.id, generation: key.generation }),
            Err(refusal) => {
                // SAFETY: a buffer `make` gave that no group holds, destroyed once; the caller is on the game thread.
                unsafe { self.buffers.destroy(handle) };
                Err(Self::refused(refusal))
            }
        }
    }

    /// Whether the group `group` of the mod `owner` is open: `STALE` for a group that was finished, dropped or is of an
    /// older world, `REFUSED` for one of another mod.
    pub fn group_check(&self, owner: usize, group: AmmlMeshGroup) -> Result<(), AmmlStatus> {
        self.groups.handle(owner, Self::key(group)).map(|_| ()).map_err(Self::refused)
    }

    /// Appends the grid `cells` of `dims` at the block position `origin`, moved by `offset` (so the grid's cells land
    /// `offset` blocks from the group's origin; the cells are still tessellated at their world positions), to the open
    /// group `group` of the mod `owner`. `REFUSED` when the group would hold more than `MAX_GROUP_VERTICES`, nothing
    /// appended; `FAILED` when the game heap or the game gave no object, and then the group is dropped, as its buffer may
    /// be part way.
    ///
    /// # Safety
    /// As `build`.
    #[allow(clippy::too_many_arguments)]
    pub unsafe fn group_append(
        &self,
        owner: usize,
        group: AmmlMeshGroup,
        builder: &MeshBuilder,
        screen: usize,
        source: usize,
        epoch: u64,
        grid: ([u32; 3], [i32; 3], &[u16]),
        offset: [i32; 3],
        blocks: &[usize],
        look: (u8, bool),
        world_block: Option<&dyn Fn([i32; 3]) -> usize>,
        heap: &MsvcHeap,
    ) -> Result<(), AmmlStatus> {
        let handle = self.groups.handle(owner, Self::key(group)).map_err(Self::refused)?;
        let origin = grid.1;
        let anchor = std::array::from_fn(|axis| origin[axis].wrapping_sub(offset[axis]));
        // SAFETY: the caller's contract; the cells move to `anchor` in the output.
        let (list, _, _) =
            unsafe { self.draw(builder, screen, source, epoch, grid, anchor, blocks, look, world_block, heap) }?;
        let Some((list, vertices)) = list else { return Ok(()) };
        // SAFETY: the group is open, so its buffer is live, on the game thread.
        let held = unsafe { MeshDataBuffer::layout(handle) }.positions.len();
        if held.saturating_add(vertices) > Self::MAX_GROUP_VERTICES {
            // SAFETY: the list is open.
            unsafe { builder.abandon(list) };
            return Err(AmmlStatus::REFUSED);
        }
        // SAFETY: the buffer is live and its vectors are on `heap`; the list is open on the live tessellator.
        let added = unsafe { MeshDataBuffer::add(heap, handle, &(*list).mesh_data) };
        // The copy is made, so the game's tessellator is free again in this frame.
        // SAFETY: the list is open.
        unsafe { builder.abandon(list) };
        if !added {
            curia::warn!("mesh group buffer could not take a grid, the group is dropped", {});
            self.group_drop(owner, group);
            return Err(AmmlStatus::FAILED);
        }
        Ok(())
    }

    /// Ends the group `group` of the mod `owner` into one mesh in the frame's tessellator of `screen`: the vertices move
    /// as a triangle list with its own index buffer. Gives the mesh handle (`None` when the group holds nothing) and its
    /// quads. The group is gone after the call, whatever it gives. `FAILED` when the game heap or the game gave no
    /// mesh.
    ///
    /// # Safety
    /// Called inside the `renderLevel` call whose screen context is `screen`, on the game thread; `heap` is the game's
    /// allocator.
    pub unsafe fn group_finish(
        &self,
        owner: usize,
        group: AmmlMeshGroup,
        builder: &MeshBuilder,
        screen: usize,
        heap: &MsvcHeap,
    ) -> Result<(Option<usize>, u32), AmmlStatus> {
        let handle = self.groups.finish(owner, Self::key(group)).map_err(Self::refused)?;
        // SAFETY: the buffer is live, off the book now, and used on the game thread only.
        let vertices = unsafe { MeshDataBuffer::layout(handle) }.positions.len();
        let made = if vertices == 0 {
            Ok(None)
        } else {
            // SAFETY: as the function's contract; the buffer is live.
            unsafe { self.mesh_of(builder, screen, handle, heap) }.map(Some)
        };
        // SAFETY: the buffer is off the book, destroyed once: empty when it moved into a mesh, else whole.
        unsafe { self.buffers.destroy(handle) };
        let quads = u32::try_from(vertices / MeshPost::QUAD).unwrap_or(u32::MAX);
        made.map(|mesh| (mesh, quads))
    }

    // Ends the vertices of the buffer at `handle` into a mesh of the frame's tessellator: a triangle list holding the
    // buffer's vertices and indices, moved in with the game's move assignment.
    //
    // Safety: as `group_finish`, with a live buffer of vertices.
    unsafe fn mesh_of(
        &self,
        builder: &MeshBuilder,
        screen: usize,
        handle: usize,
        heap: &MsvcHeap,
    ) -> Result<usize, AmmlStatus> {
        // SAFETY: the buffer is live and its vectors are on `heap`.
        let Some(indices) = (unsafe { MeshDataBuffer::triangulate(heap, handle) }) else {
            curia::warn!("mesh group has no whole quads or the index buffer could not grow", {});
            return Err(AmmlStatus::FAILED);
        };
        // SAFETY: the buffer is live.
        let vertices = unsafe { MeshDataBuffer::layout(handle) }.positions.len();
        let counted = u32::try_from(vertices).map_err(|_| AmmlStatus::FAILED)?;
        // SAFETY: the caller guarantees the running `renderLevel` call of `screen`, on its thread.
        let list = unsafe { builder.open_mode(screen, MeshBuilder::TRIANGLE_LIST, 0) }?;
        // SAFETY: the list is open on the live tessellator, whose mesh data the buffer moves into; the buffer is left
        // empty.
        unsafe {
            self.buffers.move_into(handle, &raw mut (*list).mesh_data);
            (*list).count = counted;
            Self::fit_box(list);
        }
        curia::debug!("mesh group ended", { vertices: vertices, indices: indices });
        // SAFETY: the list is open and holds the vertices and indices of the group.
        unsafe { builder.close(list) }
    }

    /// Ends the group `group` of the mod `owner` without a mesh: the group is gone and its slot free at once; its buffer
    /// is destroyed at the next `renderLevel` (`destroy_due`), so the call is safe on any thread. `STALE` for a group
    /// that is gone, `REFUSED` for one of another mod.
    pub fn group_discard(&self, owner: usize, group: AmmlMeshGroup) -> Result<(), AmmlStatus> {
        self.groups.discard(owner, Self::key(group)).map_err(Self::refused)
    }

    // Drops the group `group` of `owner` and destroys its buffer.
    fn group_drop(&self, owner: usize, group: AmmlMeshGroup) {
        if let Ok(handle) = self.groups.finish(owner, Self::key(group)) {
            // SAFETY: the buffer is off the book, destroyed once, on the game thread (the caller's).
            unsafe { self.buffers.destroy(handle) };
        }
    }

    fn key(group: AmmlMeshGroup) -> GroupKey {
        GroupKey { id: group.id, generation: group.generation }
    }

    fn refused(refusal: GroupRefusal) -> AmmlStatus {
        match refusal {
            GroupRefusal::Stale => AmmlStatus::STALE,
            GroupRefusal::NotOwner | GroupRefusal::TooMany => AmmlStatus::REFUSED,
        }
    }

    // Tessellates the grid into the frame's list and leaves it open: the cells at their world positions, the output
    // moved to `anchor` (the block position the output's origin is at). Gives the open list and its vertices (`None`,
    // nothing left open, when no cell drew a vertex), the grid indices of the skipped cells and the cells drawn.
    //
    // Safety: as `build`.
    #[allow(clippy::too_many_arguments)]
    unsafe fn draw(
        &self,
        builder: &MeshBuilder,
        screen: usize,
        source: usize,
        epoch: u64,
        grid: ([u32; 3], [i32; 3], &[u16]),
        anchor: [i32; 3],
        blocks: &[usize],
        look: (u8, bool),
        world_block: Option<&dyn Fn([i32; 3]) -> usize>,
        heap: &MsvcHeap,
    ) -> Result<Drawn, AmmlStatus> {
        let (dims, origin, cells) = grid;
        let window = CacheWindow::place(dims, cells).ok_or(AmmlStatus::INVALID_ARGUMENT)?;
        // SAFETY: the caller's contract: the game thread, a live block source of this world.
        let (tessellator, air, grass) = unsafe { self.tessellator(source, epoch) }?;
        // SAFETY: a live tessellator of this mesher, the live block source and the air block of this world.
        unsafe { self.prepare(tessellator, source, origin, air, &window, blocks) };
        let mut skipped = Vec::new();
        let mut draws: Vec<Draw> = Vec::new();
        for [x, y, z] in CacheWindow::interior(dims) {
            let index = CacheWindow::grid_index(dims, x, y, z);
            let value = cells[index];
            let Some(block) = blocks.get(usize::from(value)).copied().filter(|block| *block != air) else {
                continue;
            };
            let pos = [origin[0] + x as i32, origin[1] + y as i32, origin[2] + z as i32];
            if world_block.is_some_and(|world| world(pos) == block) {
                skipped.push(index as u32);
                continue;
            }
            // SAFETY: a live block of this world, on the game thread.
            let (passes, two_sided) = unsafe { self.describe(block) };
            draws.push((block, pos, passes, two_sided));
        }
        if draws.is_empty() {
            return Ok((None, skipped, 0));
        }
        let reserve: usize = draws
            .iter()
            .map(|(_, _, passes, two_sided)| passes.len() * Self::VERTICES_PER_PASS * if *two_sided { 2 } else { 1 })
            .sum();
        // SAFETY: the caller guarantees the running `renderLevel` call of `screen`, on its thread.
        let list = unsafe { builder.open(screen, i32::try_from(reserve).unwrap_or(i32::MAX)) }?;
        let scope = (source, grass, look.1);
        for (block, pos, passes, two_sided) in &draws {
            for layer in passes {
                // SAFETY: a live tessellator of this mesher; the list is open on the frame's tessellator.
                unsafe {
                    self.pass(tessellator, list, (*block, *pos, anchor), *layer, (look.0, scope), *two_sided, heap)
                };
            }
        }
        // SAFETY: the list is open on the live frame tessellator.
        let vertices = unsafe { Self::fit_box(list) };
        if vertices == 0 {
            // SAFETY: the list is open.
            unsafe { builder.abandon(list) };
            return Ok((None, skipped, draws.len()));
        }
        Ok((Some((list, vertices)), skipped, draws.len()))
    }

    /// The world is left: every group is dropped, and the groups' buffers and the tessellator are destroyed now on the
    /// game thread (`on_game_thread`), else at the next `renderLevel` (`destroy_due`).
    pub(crate) fn world_left(&self, on_game_thread: bool) {
        self.groups.world_left();
        if on_game_thread {
            self.destroy_groups();
        }
        let Some((tessellator, _, _, _)) = Self::lock(&self.made).take() else { return };
        if on_game_thread {
            // SAFETY: a tessellator this mesher made, taken out of the slot, on the game thread.
            unsafe { self.destroy(tessellator) };
        } else {
            *Self::lock(&self.pending) = Some(tessellator);
        }
    }

    /// The mod at `owner` is disabled: its groups are dropped; their buffers are destroyed at the next `renderLevel`.
    pub(crate) fn disable_owner(&self, owner: usize) -> usize {
        self.groups.disable_owner(owner)
    }

    /// The render device was reset or the surface lost: every group is dropped; the buffers are destroyed at the next
    /// `renderLevel`.
    pub(crate) fn device_reset(&self) -> usize {
        self.groups.device_reset()
    }

    /// Whether the water colour of a biome can be read: the entry resolved and the chunk reads give a biome.
    pub(crate) fn offers_water(&self) -> bool {
        self.tint.offers_water()
    }

    /// Whether a tessellator or a group buffer waits for `destroy_due`.
    pub(crate) fn has_due(&self) -> bool {
        Self::lock(&self.pending).is_some() || self.groups.has_due()
    }

    /// Before the game's `renderLevel`, on its thread: destroys a tessellator whose world was left off that thread and
    /// the buffers of the groups that were dropped.
    pub(crate) fn destroy_due(&self) {
        if let Some(tessellator) = Self::lock(&self.pending).take() {
            // SAFETY: a tessellator this mesher made, taken out of the slot, on the `renderLevel` thread.
            unsafe { self.destroy(tessellator) };
        }
        self.destroy_groups();
    }

    // Destroys the buffers of the groups the book dropped, each once.
    fn destroy_groups(&self) {
        for handle in self.groups.take_due() {
            // SAFETY: a buffer `make` gave, off the book, taken out of the queue so destroyed once, on the game thread.
            unsafe { self.buffers.destroy(handle) };
        }
    }

    // The tessellator of the world of `epoch`, the air block and the grass block type of that world, made now when there
    // is none for it; a tessellator of an older world that is still held is destroyed first. `FAILED` when the registry
    // gives no `minecraft:air` or the game heap no block; a world with no grass block has type 0.
    //
    // Safety: the game thread; `source` is the live block source of the local player of that world.
    unsafe fn tessellator(
        &self,
        source: usize,
        epoch: u64,
    ) -> Result<(*mut BlockTessellatorLayout, usize, usize), AmmlStatus> {
        let mut made = Self::lock(&self.made);
        if let Some((tessellator, made_in, air, grass)) = *made {
            if made_in == epoch {
                return Ok((std::ptr::with_exposed_provenance_mut(tessellator), air, grass));
            }
            *made = None;
            // SAFETY: a tessellator this mesher made, out of the slot, on the game thread.
            unsafe { self.destroy(tessellator) };
        }
        self.destroy_due();
        let Some(air) = self.registry.data_zero_state(Self::AIR)? else {
            curia::warn!("block mesher found no air block", { name: Self::AIR });
            return Err(AmmlStatus::FAILED);
        };
        let grass = match self.registry.data_zero_state(Self::GRASS) {
            // SAFETY: a live block of this world from the registry, on the game thread.
            Ok(Some(block)) => unsafe { self.tint.block_type(block) },
            _ => 0,
        };
        let block = ClientAdapter::heap_new(BlockTessellatorLayout::SIZE);
        if block.is_null() {
            curia::warn!("block tessellator not allocated", { size: BlockTessellatorLayout::SIZE });
            return Err(AmmlStatus::FAILED);
        }
        let tessellator = block.cast::<BlockTessellatorLayout>();
        // SAFETY: a block of sizeof(BlockTessellator) from the game heap, constructed once with a live region.
        unsafe { (self.ctor)(tessellator, source) };
        *made = Some((tessellator.expose_provenance(), epoch, air, grass));
        curia::info!("block tessellator made", { epoch: epoch });
        Ok((tessellator, air, grass))
    }

    // Destroys and frees a tessellator this mesher made.
    //
    // Safety: `tessellator` was made by `tessellator` and is destroyed once, on the game thread.
    unsafe fn destroy(&self, tessellator: usize) {
        let pointer = std::ptr::with_exposed_provenance_mut::<BlockTessellatorLayout>(tessellator);
        // SAFETY: the caller guarantees a constructed tessellator destroyed once; its block is the game heap block of
        // sizeof(BlockTessellator) it was made in.
        unsafe { (self.dtor)(pointer) };
        ClientAdapter::heap_delete(pointer.cast(), BlockTessellatorLayout::SIZE);
        curia::info!("block tessellator destroyed");
    }

    // Points the tessellator at `source`, sets its flags and fills its cache window: `air` at +0, in every extra block and
    // in every window cell the grid leaves empty, so no cell is read from the world.
    //
    // Safety: a live tessellator of this mesher, the live block source of this world and its air block, on the game
    // thread.
    unsafe fn prepare(
        &self,
        tessellator: *mut BlockTessellatorLayout,
        source: usize,
        origin: [i32; 3],
        air: usize,
        window: &[WindowCell],
        blocks: &[usize],
    ) {
        // SAFETY: as the function's contract.
        unsafe { (self.set_region)(tessellator, source) };
        let base = tessellator.expose_provenance();
        let cache = base + offset_of!(BlockTessellatorLayout, block_cache);
        let blocks_at = cache + offset_of!(BlockTessellatorCacheLayout, blocks);
        let extra_at = cache + offset_of!(BlockTessellatorCacheLayout, extra_blocks);
        let lights_at = cache + offset_of!(BlockTessellatorCacheLayout, light_colors);
        let write = |at: usize, value: usize| {
            // SAFETY: every address written is inside the live tessellator (offsets of the mirror).
            unsafe { std::ptr::with_exposed_provenance_mut::<usize>(at).write_unaligned(value) }
        };
        // SAFETY: the flags and the window lie inside the live tessellator; the writes are unaligned.
        unsafe {
            std::ptr::with_exposed_provenance_mut::<bool>(base + offset_of!(BlockTessellatorLayout, use_normals))
                .write(false);
            std::ptr::with_exposed_provenance_mut::<bool>(
                base + offset_of!(BlockTessellatorLayout, apply_ambient_occlusion),
            )
            .write(false);
            std::ptr::with_exposed_provenance_mut::<bool>(
                base + offset_of!(BlockTessellatorLayout, use_cached_calculation),
            )
            .write(true);
            std::ptr::with_exposed_provenance_mut::<[i32; 3]>(cache + offset_of!(BlockTessellatorCacheLayout, pos))
                .write_unaligned(origin);
        }
        write(cache + offset_of!(BlockTessellatorCacheLayout, air_block), air);
        for (index, cell) in window.iter().enumerate() {
            let block = match cell {
                WindowCell::Air => air,
                WindowCell::Palette(value) => blocks.get(usize::from(*value)).copied().unwrap_or(air),
            };
            write(blocks_at + index * size_of::<usize>(), block);
            write(extra_at + index * size_of::<usize>(), air);
            // SAFETY: the light values lie inside the live tessellator.
            unsafe {
                std::ptr::with_exposed_provenance_mut::<u16>(lights_at + index * size_of::<u16>())
                    .write_unaligned(Self::FULL_LIGHT)
            };
        }
    }

    // The render layers of `block` and whether its shape gets back faces.
    //
    // Safety: a live block of this world, on the game thread.
    unsafe fn describe(&self, block: usize) -> (Vec<i32>, bool) {
        // SAFETY: a live block (row M6: a nullable pointer).
        let graphics = unsafe { (self.graphics)(block) };
        // SAFETY: a live `BlockGraphics` holds the 4-byte shape at the entry's offset (row R1).
        let shape = unsafe { ObjectRead::value::<i32>(graphics, self.block_shape_at) }.unwrap_or(0);
        // SAFETY: a live block (row M7).
        let main = unsafe { (self.render_layer)(block) };
        // SAFETY: a live block holds its block type pointer at `mBlockType`.
        let extra = unsafe { ObjectRead::pointer(block, self.block_type_at) }
            .and_then(|block_type| {
                InventoryAdapter::virtual_fn(block_type, self.extra_layers_slot).map(|function| (block_type, function))
            })
            .map_or(0, |(block_type, function)| {
                // SAFETY: the slot of `BlockType.getExtraRenderLayers` in the vtable of a live block type (row M7).
                let function = unsafe { std::mem::transmute::<usize, ExtraLayersFn>(function) };
                // SAFETY: a live block type, on the game thread.
                unsafe { function(block_type) }
            });
        (MeshPost::passes(main, extra).collect(), MeshPost::is_two_sided(shape))
    }

    // Tessellates `block` at the world position `pos` in `layer` into the open list, moves the vertices it wrote to
    // coordinates relative to `origin` (so the 0.01 push is exact far from the world origin), tints them and applies
    // `MeshPost` to them. `look` is the alpha and the tint scope: the block source, the grass block type and whether water
    // takes the biome colour.
    //
    // Safety: a live tessellator of this mesher with its window filled, a quad list open on the live `list`, a live block
    // of this world, on the game thread; `heap` is the game's allocator.
    #[allow(clippy::too_many_arguments)]
    unsafe fn pass(
        &self,
        tessellator: *mut BlockTessellatorLayout,
        list: *mut TessellatorLayout,
        (block, pos, origin): (usize, [i32; 3], [i32; 3]),
        layer: i32,
        look: (u8, (usize, usize, bool)),
        two_sided: bool,
        heap: &MsvcHeap,
    ) {
        let base = tessellator.expose_provenance();
        // SAFETY: the render layer lies inside the live tessellator.
        unsafe {
            std::ptr::with_exposed_provenance_mut::<i32>(base + offset_of!(BlockTessellatorLayout, rendering_layer))
                .write_unaligned(layer)
        };
        // SAFETY: the list is open; the borrow ends before the game call.
        let first = unsafe { (*list).mesh_data.positions.len() };
        // SAFETY: as the function's contract; the position lives for the call; the cached calculation is on.
        unsafe { (self.tessellate)(tessellator, list, block, &raw const pos, 1) };
        // SAFETY: the list is open; the game wrote the vertices of the pass from `first` on.
        let data = unsafe { &mut (*list).mesh_data };
        let last = data.positions.len();
        if last <= first {
            return;
        }
        let positions = &mut Self::slice(&mut data.positions)[first..];
        MeshPost::shift(positions, origin);
        let centre: [f32; 3] = std::array::from_fn(|axis| (pos[axis] - origin[axis]) as f32 + 0.5);
        MeshPost::push_in(positions, centre);
        if data.colors.len() == last {
            let (alpha, scope) = look;
            let colors = &mut Self::slice(&mut data.colors)[first..];
            // SAFETY: a live block of the world of the live block source of the scope, on the game thread.
            unsafe { self.tint.apply(scope, block, pos, centre, positions, colors) };
            MeshPost::set_alpha(colors, alpha);
        }
        if two_sided {
            // SAFETY: the mesh data's vectors are the game's, on `heap`.
            match unsafe { Self::mirror(data, first, heap) } {
                // SAFETY: the list is open.
                Some(added) => unsafe { (*list).count += u32::try_from(added).unwrap_or(0) },
                None => {
                    if !self.unmirrored_logged.swap(true, Ordering::Relaxed) {
                        curia::warn!("block left one sided, its mesh vectors do not fit", {});
                    }
                }
            }
        }
    }

    // Appends the back faces of the vertices from `first` on to every per-vertex vector; gives the vertices added. `None`,
    // with the mesh data as it was, when the vertices are not whole quads, the data has indices, a vector holds a count
    // other than one per vertex (or none), or the heap cannot grow a vector.
    //
    // Safety: every vector of `data` came from `heap` (or is null); `heap` is the game's allocator.
    unsafe fn mirror(data: &mut MeshDataLayout, first: usize, heap: &MsvcHeap) -> Option<usize> {
        let last = data.positions.len();
        let added = last - first;
        if !added.is_multiple_of(MeshPost::QUAD) || !data.indices.is_empty() {
            return None;
        }
        let whole = |len: usize| len == 0 || len == last;
        let all_whole = whole(data.colors.len())
            && whole(data.normals.len())
            && whole(data.tangents.len())
            && whole(data.bone_id0s.len())
            && whole(data.pbr_texture_indices.len())
            && whole(data.mers.len())
            && whole(data.geo_type.len())
            && data.texture_uvs.iter().all(|uvs| whole(uvs.len()));
        if !all_whole {
            return None;
        }
        // SAFETY: passed on from the caller.
        let room = unsafe {
            Self::reserve(heap, &mut data.positions, last + added)
                && Self::reserve(heap, &mut data.colors, last + added)
                && Self::reserve(heap, &mut data.normals, last + added)
                && Self::reserve(heap, &mut data.tangents, last + added)
                && Self::reserve(heap, &mut data.bone_id0s, last + added)
                && Self::reserve(heap, &mut data.pbr_texture_indices, last + added)
                && Self::reserve(heap, &mut data.mers, last + added)
                && Self::reserve(heap, &mut data.geo_type, last + added)
                && data.texture_uvs.iter_mut().all(|uvs| Self::reserve(heap, uvs, last + added))
        };
        if !room {
            return None;
        }
        // SAFETY: the room is reserved for every vector that is copied, so no append grows a vector.
        unsafe {
            Self::append(heap, &mut data.positions, first, |vertex| vertex);
            Self::append(heap, &mut data.colors, first, |color| color);
            Self::append(heap, &mut data.normals, first, MeshPost::turn_normal);
            Self::append(heap, &mut data.tangents, first, MeshPost::turn_normal);
            Self::append(heap, &mut data.bone_id0s, first, |value| value);
            Self::append(heap, &mut data.pbr_texture_indices, first, |value| value);
            Self::append(heap, &mut data.mers, first, |value| value);
            Self::append(heap, &mut data.geo_type, first, |value| value);
            for uvs in &mut data.texture_uvs {
                Self::append(heap, uvs, first, |uv| uv);
            }
        }
        Some(added)
    }

    // Makes room for `total` elements in a vector that holds some; true for an empty vector, which is left out.
    //
    // Safety: as `mirror`.
    unsafe fn reserve<T: Copy>(heap: &MsvcHeap, vec: &mut MsvcVec<T>, total: usize) -> bool {
        // SAFETY: passed on from the caller.
        vec.is_empty() || unsafe { GameVector::reserve(heap, vec, total) }.is_ok()
    }

    // Appends the back faces of the elements from `first` on to a vector that holds some.
    //
    // Safety: as `mirror`, with the room reserved.
    unsafe fn append<T: Copy>(heap: &MsvcHeap, vec: &mut MsvcVec<T>, first: usize, map: impl Fn(T) -> T) {
        if vec.is_empty() {
            return;
        }
        let Some(back) = MeshPost::reversed(&vec.as_slice()[first..], map) else { return };
        // SAFETY: passed on from the caller.
        let _ = unsafe { GameVector::extend_with(heap, vec, back.len(), |index| back[index]) };
    }

    // Fits the mesh data box of the open list to its positions (already relative to the grid origin) and gives the vertex
    // count.
    //
    // Safety: a list is open on the live `list`.
    unsafe fn fit_box(list: *mut TessellatorLayout) -> usize {
        // SAFETY: as the function's contract.
        let data = unsafe { &mut (*list).mesh_data };
        let positions = Self::slice(&mut data.positions);
        data.aabb = MeshPost::fit_aabb(positions);
        positions.len()
    }

    // The elements of a game vector, for writing.
    fn slice<T>(vec: &mut MsvcVec<T>) -> &mut [T] {
        let [first, last, _] = vec.raw_parts();
        if first.is_null() {
            return &mut [];
        }
        // SAFETY: the game's vector holds `last - first` initialised elements from `first`; the borrow of `vec` keeps
        // the vector from changing while the slice lives.
        unsafe { std::slice::from_raw_parts_mut(first, last.offset_from(first).unsigned_abs()) }
    }

    fn note_time(&self, cells: usize, quads: u32, started: Instant) {
        let us = u32::try_from(started.elapsed().as_micros()).unwrap_or(u32::MAX);
        let first = !self.logged.swap(true, Ordering::Relaxed);
        let slowest = self.slowest_us.fetch_max(us, Ordering::Relaxed);
        if first || (us > Self::SLOW_US && us > slowest) {
            curia::info!("mesher build time", { cells: cells, quads: quads, us: us });
        }
    }

    // The slots hold plain addresses, so a lock poisoned by a panic is recovered.
    fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
        mutex.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}
