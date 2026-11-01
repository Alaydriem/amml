/// The `DimensionType` struct of the game: one `int`. A member function returns it through a hidden pointer.
#[repr(C)]
pub(crate) struct DimensionType {
    pub(crate) value: i32,
}
