use crate::structs::{WalkBlock, WalkedColumn};

/// The blocks of one column, read by block y. `None` is a block the game gave no readable type for.
pub trait ColumnBlocks {
    fn block(&mut self, y: i32) -> Option<WalkBlock>;
}

/// The walk of one column of a chunk from a camera height down to the block the map shows.
///
/// The chunk holds the blocks from `min_y` up to, but not including, `max_y`. The walk reads nothing outside that
/// range. A block that cannot be read is air, and a block with no readable material is solid. The rule:
///
/// - Camera below the chunk: nothing is shown.
/// - Camera above the chunk: start at the column height (the top of the chunk when the height is outside it) and go
///   up to the top of the chunk to the first air; the block below that air is shown. When there is no air, the top
///   block is shown.
/// - Camera in air: when the column height is below the camera, as above, from the height up to the camera. Else the
///   first block below the camera that is not air is shown; all air is void.
/// - Camera in a block that is not solid (water, plants): that block is shown.
/// - Camera in a solid block (cave mode): the first block below that is not solid is shown. At the first air, the
///   first block below the air that is not air is shown, or void when there is none. A solid run to the bottom shows
///   nothing.
/// - With transparent water, water that is to be shown gives its depth and the first block under it that is not
///   water or air as the shown block. Water down to the bottom shows the water itself, with the depth to the bottom
///   of the chunk.
/// - A placeholder block sets the retry flag.
pub struct ColumnWalk {
    min_y: i32,
    max_y: i32,
    transparent_water: bool,
}

// A block read, and what the walk asks of it.
struct Seen {
    block: Option<WalkBlock>,
}

impl Seen {
    fn is_air(&self) -> bool {
        self.block.is_none_or(|block| block.is_air)
    }

    fn is_water(&self) -> bool {
        self.block.is_some_and(|block| block.is_water && !block.is_air)
    }

    fn is_solid(&self) -> bool {
        self.block.is_none_or(|block| block.is_solid)
    }

    fn id(&self) -> u32 {
        self.block.map_or(0, |block| block.id)
    }
}

// The state of one walk.
struct Walk<'b, B: ColumnBlocks> {
    blocks: &'b mut B,
    min_y: i32,
    max_y: i32,
    transparent_water: bool,
    column: WalkedColumn,
}

impl ColumnWalk {
    /// A walk over a chunk that holds the block ys `min_y` up to but not including `max_y`.
    pub fn new(min_y: i32, max_y: i32, transparent_water: bool) -> Self {
        Self { min_y, max_y, transparent_water }
    }

    /// Walks the column from `camera_y`. `height` is the block y of the column from the heightmap.
    pub fn walk(&self, blocks: &mut impl ColumnBlocks, camera_y: i32, height: i32) -> WalkedColumn {
        let mut walk = Walk {
            blocks,
            min_y: self.min_y,
            max_y: self.max_y,
            transparent_water: self.transparent_water,
            column: WalkedColumn::default(),
        };
        if self.max_y > self.min_y {
            walk.pick(camera_y, height);
        }
        walk.column
    }
}

impl<B: ColumnBlocks> Walk<'_, B> {
    fn top(&self) -> i32 {
        self.max_y - 1
    }

    // The block at `y`; a y outside the chunk is never asked of the game.
    fn seen(&mut self, y: i32) -> Seen {
        if y < self.min_y || y >= self.max_y {
            return Seen { block: None };
        }
        let block = self.blocks.block(y);
        if block.is_some_and(|block| block.is_placeholder) {
            self.column.retry = true;
        }
        Seen { block }
    }

    fn show(&mut self, y: i32, seen: &Seen) {
        self.column.shown = seen.id();
        self.column.shown_y = y;
    }

    // Shows the block at `y`; for transparent water, its depth and the block below it.
    fn show_kind(&mut self, y: i32, seen: &Seen) {
        if !(self.transparent_water && seen.is_water()) {
            self.show(y, seen);
            return;
        }
        self.column.water = seen.id();
        for below in (self.min_y..y).rev() {
            let under = self.seen(below);
            if !under.is_water() && !under.is_air() {
                self.column.water_depth = (y - below).min(255) as u8;
                self.show(below, &under);
                return;
            }
        }
        self.column.water_depth = (y - self.min_y).clamp(0, 255) as u8;
        self.show(y, seen);
    }

    // The block below the first air at or above `from`, up to `to`; the block at `to` when none is air.
    fn first_below_air(&mut self, from: i32, to: i32) {
        for y in from..=to {
            if !self.seen(y).is_air() {
                continue;
            }
            if y <= self.min_y {
                self.column.void = true;
                return;
            }
            let below = self.seen(y - 1);
            self.show_kind(y - 1, &below);
            return;
        }
        let seen = self.seen(to);
        self.show_kind(to, &seen);
    }

    fn pick(&mut self, camera: i32, height: i32) {
        let top = self.top();
        if camera < self.min_y {
            return;
        }
        if camera > top {
            let from = if height < self.min_y || height > top { top } else { height };
            self.first_below_air(from, top);
            return;
        }
        let at_camera = self.seen(camera);
        if at_camera.is_air() {
            if height < camera {
                self.first_below_air(height, camera);
                return;
            }
            for y in (self.min_y..=camera).rev() {
                let seen = self.seen(y);
                if !seen.is_air() {
                    self.show_kind(y, &seen);
                    return;
                }
            }
            self.column.void = true;
            return;
        }
        if !at_camera.is_solid() {
            self.show_kind(camera, &at_camera);
            return;
        }
        for y in (self.min_y..camera).rev() {
            let seen = self.seen(y);
            if seen.is_air() {
                for below in (self.min_y..y).rev() {
                    let under = self.seen(below);
                    if !under.is_air() {
                        self.show_kind(below, &under);
                        return;
                    }
                }
                self.column.void = true;
                return;
            }
            if !seen.is_solid() {
                self.show_kind(y, &seen);
                return;
            }
        }
    }
}
