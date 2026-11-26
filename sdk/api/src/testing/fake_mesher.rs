use amml_abi::{AmmlBlockHandle, AmmlStatus};

/// The block mesher of a `FakeGame`: the loader's checks of a grid (a side above 20 is `REFUSED`, cells that are not the
/// product of the dims or index past the palette are `INVALID_ARGUMENT`) and a stand-in for the game's tessellator that
/// draws every non-empty inner cell as a full block of six quads.
pub(crate) struct FakeMesher;

impl FakeMesher {
    // The window side of the game's block cache.
    const MAX_DIM: u32 = 20;
    const EMPTY: u16 = u16::MAX;
    // The quads of a full block.
    const QUADS_PER_CELL: u32 = 6;

    /// The cells drawn and the quads of a build of `cells` of `dims` with `palette`.
    pub(crate) fn build(dims: [u32; 3], cells: &[u16], palette: &[AmmlBlockHandle]) -> Result<(u32, u32), AmmlStatus> {
        if dims.iter().any(|side| *side > Self::MAX_DIM) {
            return Err(AmmlStatus::REFUSED);
        }
        if dims.iter().map(|side| *side as usize).product::<usize>() != cells.len() {
            return Err(AmmlStatus::INVALID_ARGUMENT);
        }
        if cells.iter().any(|cell| *cell != Self::EMPTY && usize::from(*cell) >= palette.len()) {
            return Err(AmmlStatus::INVALID_ARGUMENT);
        }
        let inner = |value: u32, side: u32| value >= 1 && value + 1 < side;
        let mut drawn = 0u32;
        for (index, cell) in cells.iter().enumerate() {
            let index = index as u32;
            let x = index % dims[0];
            let z = (index / dims[0]) % dims[2];
            let y = index / (dims[0] * dims[2]);
            if *cell != Self::EMPTY && inner(x, dims[0]) && inner(y, dims[1]) && inner(z, dims[2]) {
                drawn += 1;
            }
        }
        Ok((drawn, drawn * Self::QUADS_PER_CELL))
    }
}
