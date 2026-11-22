/// What one cell of the block tessellator's cache window holds for a mesher build: air, or the palette index of a grid
/// cell.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WindowCell {
    Air,
    Palette(u16),
}
