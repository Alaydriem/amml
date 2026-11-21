/// One block as the column walk sees it. A block the game gave no readable type for is air; one with no readable
/// material is solid.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WalkBlock {
    /// The block type id (`BlockTypeBook`).
    pub id: u32,
    pub is_air: bool,
    pub is_water: bool,
    /// The client has not received the block yet.
    pub is_placeholder: bool,
    pub is_solid: bool,
}
