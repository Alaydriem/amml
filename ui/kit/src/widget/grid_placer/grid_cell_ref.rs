/// One cell of a grid placer, counted from 0 at the top left.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct GridCellRef {
    pub column: u16,
    pub row: u16,
}

impl GridCellRef {
    pub fn new(column: u16, row: u16) -> Self {
        Self { column, row }
    }
}
