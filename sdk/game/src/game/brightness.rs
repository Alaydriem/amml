/// A `BrightnessPair` of the game: the light of one block position, each 0 to 15. The game stores the sky light
/// first and the block light second.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(C)]
pub struct Brightness {
    pub sky: u8,
    pub block: u8,
}
