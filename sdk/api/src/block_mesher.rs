use std::marker::PhantomData;

use amml_abi::{
    AmmlBlockGrid, AmmlBlockHandle, AmmlBlockPos, AmmlMeshBuilt, AmmlMesherOptions, AmmlRenderTable, AmmlStr,
};

use crate::{ApiError, BlockGrid, BlockPos, MeshBuilt, MeshGroup, MeshId, MesherOptions};

/// The block mesher of one `WorldRendered` dispatch: builds a grid of blocks into a mesh with the game's own block
/// tessellator, so a ghost looks like the blocks it stands for (connected fences, stairs, plants). It is built only from
/// the event of a running listener call (`WorldRenderedExt::mesher`) and borrows it, so it cannot outlive the call; the
/// loader refuses a call outside the dispatch. The mesh is a mesh of the canvas, drawn with the blend-block material and
/// the terrain atlas (`MeshStyle::GHOST`), and freed as every mesh of the canvas is.
///
/// Blocks whose tint method asks for a biome colour (grass, foliage) are tinted at their world position. Water takes the
/// biome colour only with `MesherOptions::water_tint` and `Area::RENDER_WATER_TINT`; else it keeps the texture's grey.
pub struct BlockMesher<'f> {
    table: AmmlRenderTable,
    frame: u64,
    module: &'static str,
    _event: PhantomData<&'f ()>,
}

impl<'f> BlockMesher<'f> {
    /// The most cells of a grid on each axis, border included: the side of the game's block cache window (20 on both gate
    /// builds).
    pub const MAX_DIM: u32 = 20;

    pub(crate) fn new(table: AmmlRenderTable, frame: u64, module: &'static str) -> Self {
        Self { table, frame, module, _event: PhantomData }
    }

    /// Builds `grid` with `options` into a mesh of this mod. Each inner cell is drawn in every render layer its block
    /// needs; its faces move 0.01 into its cell along their normal, so a real block over a ghost block always shows (a
    /// face whose plane passes through the cell centre does not move: crossed plants, the step of a stair, the top of a
    /// bottom slab); crossed plants, vines and chains are drawn from both sides. The skipped cells are listed only with
    /// `MesherOptions::skip_matching_world`. `Err(Refused)` for a side above `MAX_DIM`; `Err(Loader(INVALID_ARGUMENT))`
    /// for cells that are not the product of `dims` or a cell index past the palette; `Err(Stale)` for a palette handle
    /// of an earlier world; `Err(NotAvailable)` from a loader without `Area::RENDER_BLOCK_MESHER`; `Err(Loader(FAILED))`
    /// when the game made no mesh.
    #[allow(unsafe_code)]
    pub fn build(&self, grid: &BlockGrid<'_>, options: &MesherOptions) -> Result<MeshBuilt, ApiError> {
        let build = self.table.mesher_build.ok_or(ApiError::NotAvailable)?;
        let (raw, _palette) = Self::raw_grid(grid)?;
        let options = AmmlMesherOptions::from(options);
        let mut out = AmmlMeshBuilt::default();
        // The skipped cells are listed only with the option: no buffer is made for a build without it.
        let mut skipped = if options.skip_matching_world != 0 { vec![0u32; grid.cells.len()] } else { Vec::new() };
        let skipped_cap = u32::try_from(skipped.len()).map_err(|_| ApiError::Refused)?;
        // SAFETY: the table function is the loader's; the module id, the grid with its cells and palette (kept alive by
        // `_palette`), and the options live for the call; `out` is writable and `skipped` holds `skipped_cap` writable
        // values (null with a cap of 0 is allowed).
        let status = unsafe {
            build(
                AmmlStr::from_str(self.module),
                self.frame,
                &raw const raw,
                &raw const options,
                &raw mut out,
                if skipped.is_empty() { std::ptr::null_mut() } else { skipped.as_mut_ptr() },
                skipped_cap,
            )
        };
        ApiError::check(status)?;
        skipped.truncate(out.skipped_count as usize);
        Ok(MeshBuilt { mesh: MeshId::from(out.mesh), quads: out.quads, skipped })
    }

    /// Opens a group whose slots are `slot_size` blocks. Grids appended to it (`append`) become one mesh at `finish`, which
    /// is the way to build more quads than one mesh holds. The group lives across dispatches of the same world: keep it
    /// and append to it in later `WorldRendered` calls. Dropping the group without `finish` discards it and gives its slot
    /// back. `Err(Loader(INVALID_ARGUMENT))` for a slot side of 0;
    /// `Err(Refused)` when this mod holds `MeshGroup::MAX_OPEN` groups;
    /// `Err(NotAvailable)` from a loader without `Area::RENDER_BLOCK_MESHER`.
    #[allow(unsafe_code)]
    pub fn begin_group(&self, slot_size: [u32; 3]) -> Result<MeshGroup, ApiError> {
        let begin = self.table.mesher_group_begin.ok_or(ApiError::NotAvailable)?;
        let mut out = amml_abi::AmmlMeshGroup::default();
        // SAFETY: the table function is the loader's; the module id and the slot size live for the call; `out` is
        // writable.
        let status = unsafe { begin(AmmlStr::from_str(self.module), self.frame, &raw const slot_size, &raw mut out) };
        ApiError::check(status)?;
        Ok(MeshGroup::from_raw(out, self.table.mesher_group_discard, self.module))
    }

    /// Appends `grid` with `options` to `group`, with the grid's cells placed `offset` blocks from the group's origin: a
    /// cell of the grid at the world position `p` lands at `p - grid.origin + offset` in the mesh that `finish` makes. The
    /// cells are tessellated at their world positions, so their neighbours and biome tint are as in `build`. With
    /// `MesherOptions::skip_matching_world` the cells whose world block matches are left out; their indices are not
    /// reported. Errors as `build`, and `Err(Stale)` for a group of an earlier world, one the loader dropped or one of a
    /// disabled mod; `Err(Refused)` when the group would hold more than 65,536 quads, with the group as it was;
    /// `Err(Loader(FAILED))` when the game made no object, and the group is then dropped.
    #[allow(unsafe_code)]
    pub fn append(
        &self,
        group: &mut MeshGroup,
        grid: &BlockGrid<'_>,
        offset: BlockPos,
        options: &MesherOptions,
    ) -> Result<(), ApiError> {
        let append = self.table.mesher_group_append.ok_or(ApiError::NotAvailable)?;
        let (raw, _palette) = Self::raw_grid(grid)?;
        let offset = AmmlBlockPos::from(offset);
        let options = AmmlMesherOptions::from(options);
        // SAFETY: the table function is the loader's; the module id, the grid with its cells and palette (kept alive by
        // `_palette`), the offset and the options live for the call.
        let status = unsafe {
            append(
                AmmlStr::from_str(self.module),
                self.frame,
                group.raw(),
                &raw const raw,
                &raw const offset,
                &raw const options,
            )
        };
        ApiError::check(status)
    }

    /// Ends `group` into one mesh of this mod (drawn and freed as the mesh of a build); the group is gone, whatever the
    /// result. `MeshBuilt::skipped` is empty. A group that holds nothing gives `quads` 0 and a mesh id of no mesh, which
    /// every draw refuses with `Stale`. Errors: `Err(Stale)` for a group of an earlier world or one the loader dropped;
    /// `Err(Loader(FAILED))` when the game made no mesh.
    #[allow(unsafe_code)]
    pub fn finish(&self, group: MeshGroup) -> Result<MeshBuilt, ApiError> {
        let finish = self.table.mesher_group_finish.ok_or(ApiError::NotAvailable)?;
        let mut out = AmmlMeshBuilt::default();
        // SAFETY: the table function is the loader's; the module id lives for the call; `out` is writable.
        let status = unsafe { finish(AmmlStr::from_str(self.module), self.frame, group.into_raw(), &raw mut out) };
        ApiError::check(status)?;
        Ok(MeshBuilt { mesh: MeshId::from(out.mesh), quads: out.quads, skipped: Vec::new() })
    }

    // The ABI grid of `grid` and the palette it points to, which the caller keeps alive for the call.
    fn raw_grid(grid: &BlockGrid<'_>) -> Result<(AmmlBlockGrid, Vec<AmmlBlockHandle>), ApiError> {
        let palette: Vec<AmmlBlockHandle> = grid.palette.iter().map(|handle| handle.raw()).collect();
        let raw = AmmlBlockGrid {
            dims: grid.dims,
            origin: grid.origin.into(),
            cells: grid.cells.as_ptr(),
            cell_count: u32::try_from(grid.cells.len()).map_err(|_| ApiError::Refused)?,
            palette: palette.as_ptr(),
            palette_len: u32::try_from(palette.len()).map_err(|_| ApiError::Refused)?,
        };
        Ok((raw, palette))
    }
}
