use amml_abi::AmmlRayOptions;

/// How `BlockSource::raycast` walks.
///
/// `outline` clips each candidate block against its selection outline (what the game draws around the looked-at
/// block); without it (the default) against its collision shape. A plant or a torch has an empty collision shape, so
/// with `outline` false it is never hit, whatever `every_block` is.
///
/// `every_block` makes every block a candidate; without it the walk skips the blocks whose material is not solid
/// (plants, torches, water), which needs `Area::WORLD_CHUNK_READ` too. The two filters give different hits only when
/// `outline` is true: then a flower is hit with `every_block` true and passed through with `every_block` false.
///
/// `max_steps` is the most blocks the walk visits; 0 is the game's own count (200).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RayOptions {
    pub every_block: bool,
    pub outline: bool,
    pub max_steps: u32,
}

impl From<RayOptions> for AmmlRayOptions {
    fn from(options: RayOptions) -> Self {
        Self {
            every_block: u8::from(options.every_block),
            outline: u8::from(options.outline),
            _reserved: [0; 2],
            max_steps: options.max_steps,
        }
    }
}
