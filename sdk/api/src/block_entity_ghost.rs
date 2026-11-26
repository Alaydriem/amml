use amml_abi::AmmlBlockEntityGhost;

/// A block entity ghost (`BlockEntities::create`): the game's own block actor of a block, which the loader owns and
/// draws with the game's block actor renderer (`BlockEntityPassExt::draw_block_entity`). It is bound to this mod and the
/// world it was made in, and not `Copy` or `Clone`: `BlockEntities::free` takes it by value. The loader frees a ghost
/// when the world is left, when a render device reset or surface loss is seen and when the mod is disabled; any use after
/// that gives `Err(Stale)`. A mod holds at most `MAX_LIVE` ghosts.
#[derive(Debug, PartialEq, Eq)]
pub struct BlockEntityGhost {
    raw: AmmlBlockEntityGhost,
}

impl BlockEntityGhost {
    /// The most live ghosts a mod holds.
    pub const MAX_LIVE: usize = 256;

    pub(crate) fn from_raw(raw: AmmlBlockEntityGhost) -> Self {
        Self { raw }
    }

    pub(crate) fn raw(&self) -> AmmlBlockEntityGhost {
        self.raw
    }
}
