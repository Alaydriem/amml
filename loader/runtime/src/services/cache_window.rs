use crate::WindowCell;

/// The block tessellator's cache window as a block mesher build fills it: 20 cells on each axis from the corner `mPos`,
/// cell (x, y, z) at index x * 400 + y * 20 + z (the order both gate builds index the window with, survey section 7, row
/// M4). The mod's grid starts at the window corner; its empty cells and every window cell outside it are `Air`, which the
/// loader writes as the real air block (not the placeholder the game reads from the world), so the ghost cells join each
/// other and not the world.
///
/// Grid cells are in the order of `BlockGrid`: x first, then z, then y. The grid has one cell of border on each side; only
/// the cells inside it are drawn.
pub struct CacheWindow;

impl CacheWindow {
    /// The cells of the window on each axis.
    pub const SIDE: u32 = 20;
    /// The cells of the window.
    pub const CELLS: usize = 8000;
    /// The palette index of an empty grid cell.
    pub const EMPTY: u16 = u16::MAX;

    /// The window index of cell (x, y, z), counted from the window corner.
    pub fn index(x: u32, y: u32, z: u32) -> usize {
        (x * Self::SIDE * Self::SIDE + y * Self::SIDE + z) as usize
    }

    /// The index in a grid of `dims` of cell (x, y, z).
    pub fn grid_index(dims: [u32; 3], x: u32, y: u32, z: u32) -> usize {
        ((y * dims[2] + z) * dims[0] + x) as usize
    }

    /// Whether a grid of `dims` with `cell_count` cells fits the window: each side at most `SIDE` and the cells the product
    /// of the sides.
    pub fn fits(dims: [u32; 3], cell_count: usize) -> bool {
        let sides = dims.iter().all(|side| *side <= Self::SIDE);
        sides && dims.iter().map(|side| *side as usize).product::<usize>() == cell_count
    }

    /// The window for the grid `cells` of `dims`, `CELLS` long; `None` when the grid does not fit (`fits`).
    pub fn place(dims: [u32; 3], cells: &[u16]) -> Option<Vec<WindowCell>> {
        if !Self::fits(dims, cells.len()) {
            return None;
        }
        let mut window = vec![WindowCell::Air; Self::CELLS];
        for y in 0..dims[1] {
            for z in 0..dims[2] {
                for x in 0..dims[0] {
                    let value = cells[Self::grid_index(dims, x, y, z)];
                    if value != Self::EMPTY {
                        window[Self::index(x, y, z)] = WindowCell::Palette(value);
                    }
                }
            }
        }
        Some(window)
    }

    /// The cells of a grid of `dims` that are drawn: those inside the one-cell border, y first, then z, then x.
    pub fn interior(dims: [u32; 3]) -> impl Iterator<Item = [u32; 3]> {
        let inner = |side: u32| 1..side.saturating_sub(1);
        inner(dims[1]).flat_map(move |y| inner(dims[2]).flat_map(move |z| inner(dims[0]).map(move |x| [x, y, z])))
    }
}
