/// What the Vibrant Visuals static light pass does for one subchunk.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VvPlan {
    /// Run it as the game does.
    Normal,
    /// Skip it, so the raised vertex light stays.
    Skip,
    /// Run it with the held block added as an emitter at the world block `position`.
    Emit { block: usize, position: [i32; 3], emission: u8 },
}
