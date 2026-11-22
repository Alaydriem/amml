/// The game side of one block entity ghost: the `std::shared_ptr<BlockActor>` the loader holds (the actor and its
/// control block), the block it was made for, the skull pose it is drawn with (face, turn in sixteenths), and whether
/// the loader made it the lead of a large chest (its partner pointer then names the actor itself, and is cleared before
/// the release).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GhostEntity {
    pub actor: usize,
    pub control: usize,
    pub block: usize,
    pub skull: Option<(u8, u8)>,
    pub chest_paired: bool,
}
