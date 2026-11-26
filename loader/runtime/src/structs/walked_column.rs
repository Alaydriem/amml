/// What the walk of one column found. A block type id of 0 is none.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct WalkedColumn {
    /// The block the map shows, and its block y.
    pub shown: u32,
    pub shown_y: i32,
    /// The water above the shown block, and its depth (255 at most); 0 for none.
    pub water: u32,
    pub water_depth: u8,
    /// Nothing but air down to the bottom of the chunk.
    pub void: bool,
    /// A placeholder block was met: the column is read again later.
    pub retry: bool,
}
