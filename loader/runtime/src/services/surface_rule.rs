use crate::structs::SurfaceHit;

/// The blocks and the light of one column, read by block y. `Block` is what a read of one block gives (an address, a
/// shape); the questions about a block take it back, so a block is read once.
pub trait SurfaceColumn {
    type Block: Copy;

    /// The block at `y`. `None` when it cannot be read (not readable, or a block the client has not received yet):
    /// the scan then gives up.
    fn block(&mut self, y: i32) -> Option<Self::Block>;

    /// Whether `block` is air.
    fn is_air(&mut self, block: Self::Block) -> bool;

    /// Whether `block`, at `y`, has an empty collision shape (flowers, torches, short grass).
    fn has_empty_collision(&mut self, block: Self::Block, y: i32) -> bool;

    /// Whether `block` gives full support on its top face.
    fn gives_full_top_support(&mut self, block: Self::Block) -> bool;

    /// The raw light at `y`: sky light, then block light. `None` when the game gives none.
    fn light(&mut self, y: i32) -> Option<(u8, u8)>;
}

/// Where a mob can stand in one column of a chunk.
///
/// A surface is at `y` when the block at `y` gives full support on its top face and the block at `y + 1` is air or has
/// an empty collision shape. The light of the surface is the raw sky and block light at `y + 1`, with no sky darken.
/// Surfaces come lowest first.
///
/// The chunk holds the blocks from `min_y` up to `top_y`, both included (`top_y` is the highest block y of the chunk).
/// The scan reads no block below `min_y` and none above `top_y`, so the highest surface is at `top_y - 1`, and the block
/// above it is still inside the chunk.
pub struct SurfaceRule {
    min_y: i32,
    top_y: i32,
}

impl SurfaceRule {
    pub fn new(min_y: i32, top_y: i32) -> Self {
        Self { min_y, top_y }
    }

    /// Finds the surfaces of `column` with a block y from `y_min` to `y_max` (both included), lowest first, and gives
    /// each to `emit`, at most `limit` of them. Gives the number found. `None` when a block could not be read or the
    /// game gave no light for a surface.
    pub fn scan<C: SurfaceColumn>(
        &self,
        column: &mut C,
        y_min: i32,
        y_max: i32,
        limit: usize,
        mut emit: impl FnMut(SurfaceHit),
    ) -> Option<usize> {
        let low = y_min.max(self.min_y);
        let high = y_max.min(self.top_y.saturating_sub(1));
        if limit == 0 || low > high {
            return Some(0);
        }
        let mut count = 0;
        let mut block = column.block(low)?;
        for y in low..=high {
            if count >= limit {
                break;
            }
            let above = column.block(y + 1)?;
            let passable = column.is_air(above) || column.has_empty_collision(above, y + 1);
            if passable && column.gives_full_top_support(block) {
                let (sky_light, block_light) = column.light(y + 1)?;
                emit(SurfaceHit { y, block_light, sky_light });
                count += 1;
            }
            block = above;
        }
        Some(count)
    }
}
