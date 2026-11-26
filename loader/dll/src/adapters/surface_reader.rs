use amml_runtime::SurfaceColumn;

use crate::{ChunkAdapter, ChunkBlockPos, SurfaceAdapter, WorldBlockPos};

/// One column of a live chunk, for the `SurfaceRule`: the blocks through `LevelChunk::getBlock`, the shape of their
/// types through the block type reader, and the support, collision and light calls of the surface adapter.
///
/// A block the client has not received yet (the placeholder block), and a block the game gave no address for, make the
/// scan give up: the answer would be wrong, and the mod asks again later.
pub struct SurfaceReader<'a> {
    pub chunks: &'a ChunkAdapter,
    pub surfaces: &'a SurfaceAdapter,
    pub chunk: usize,
    pub source: usize,
    // The column in blocks.
    pub x: i32,
    pub z: i32,
    pub min_y: i32,
    pub epoch: u64,
}

impl SurfaceReader<'_> {
    fn world(&self, y: i32) -> WorldBlockPos {
        WorldBlockPos { x: self.x, y, z: self.z }
    }
}

impl SurfaceColumn for SurfaceReader<'_> {
    // The address of the block, and whether it is air.
    type Block = (usize, bool);

    fn block(&mut self, y: i32) -> Option<Self::Block> {
        let relative = i16::try_from(y.checked_sub(self.min_y)?).ok()?;
        let pos = ChunkBlockPos { x: (self.x & 15) as u8, z: (self.z & 15) as u8, y: relative };
        // SAFETY: the chunk is live and the scan runs on the game thread (`ChunkAdapter::scan_surfaces`' contract).
        let block = unsafe { self.chunks.block_at(self.chunk, pos) };
        // SAFETY: the game gives a live Block, or null.
        let shape = unsafe { self.chunks.types().walk_block(block, self.epoch) }?;
        (!shape.is_placeholder).then_some((block, shape.is_air))
    }

    fn is_air(&mut self, (_, is_air): Self::Block) -> bool {
        is_air
    }

    fn has_empty_collision(&mut self, (block, _): Self::Block, y: i32) -> bool {
        // SAFETY: the block is live (this scan read it) and the source is the live block source.
        unsafe { self.surfaces.has_empty_collision(block, self.source, &self.world(y)) }
    }

    fn gives_full_top_support(&mut self, (block, _): Self::Block) -> bool {
        // SAFETY: the block is live (this scan read it), on the game thread.
        unsafe { self.surfaces.gives_full_top_support(block) }
    }

    fn light(&mut self, y: i32) -> Option<(u8, u8)> {
        // SAFETY: the source is the live block source, on the game thread.
        Some(unsafe { self.surfaces.light(self.source, &self.world(y)) })
    }
}
