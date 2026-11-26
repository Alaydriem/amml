use amml_abi::{AmmlBlockPos, AmmlGhostEntityOptions, AmmlSkullPose};

use crate::{BlockPos, SkullPose};

/// The options of a block entity ghost (`BlockEntities::create`).
///
/// `chest_pair` makes a chest ghost the lead of a large chest whose other half is at this position relative to the ghost:
/// one block along x or z at the same height (`BlockPos { x: 0, y: 0, z: 1 }` for a partner one block south). It needs
/// `Area::RENDER_CHEST_PAIR`, and the block must be a chest. `skull` draws a skull ghost with the face it stands on or
/// hangs from and its turn on the floor (0 to 15); it needs `Area::RENDER_SKULL_POSE`, the block must be a skull, and
/// the face must be one the game numbers (`Down` to `East`), else `Err(Loader(INVALID_ARGUMENT))`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct GhostEntityOptions {
    pub chest_pair: Option<BlockPos>,
    pub skull: Option<SkullPose>,
}

impl From<&GhostEntityOptions> for AmmlGhostEntityOptions {
    fn from(options: &GhostEntityOptions) -> Self {
        Self {
            has_chest_pair: u8::from(options.chest_pair.is_some()),
            has_skull: u8::from(options.skull.is_some()),
            skull: options.skull.map(AmmlSkullPose::from).unwrap_or_default(),
            _reserved: [0; 4],
            chest_pair: options.chest_pair.map(AmmlBlockPos::from).unwrap_or_default(),
        }
    }
}
