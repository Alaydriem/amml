use amml_abi::AmmlBlockPos;

/// One block change the game made off the game thread, kept for the next frame: the dimension of the block source,
/// the position and layer, and the addresses of the new and the old `Block`. The addresses are read to block type ids
/// on the game thread when the change is raised, so no game memory is read off it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BlockChange {
    pub dimension: i32,
    pub pos: AmmlBlockPos,
    pub layer: u32,
    pub new_block: usize,
    pub old_block: usize,
}
