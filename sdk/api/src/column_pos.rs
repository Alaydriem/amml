/// A column of the world, in blocks (not chunks): the block x and z. A column has one block at each y.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct ColumnPos {
    pub x: i32,
    pub z: i32,
}
