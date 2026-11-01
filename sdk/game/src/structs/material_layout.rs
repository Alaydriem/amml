use crate::{EntryId, EntryNeed, GameEntries, GameError};

/// Which `BlockType` mirror the game build has, from the `BlockType.mMaterialInline` entry: the Material held as a
/// pointer ([`crate::game::BlockTypePointerLayout`]) or by value ([`crate::game::BlockTypeValueLayout`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MaterialLayout {
    /// `BlockType` holds a `Material const&` stored as a pointer; the solid flag is a bool byte of the Material
    /// (26.51). The entry value is 0.
    Pointer,
    /// `BlockType` holds the Material by value; the solid flag is a bit of its flag byte (26.60). The entry value
    /// is 1.
    Value,
}

impl MaterialLayout {
    /// The feature of the layout entry.
    pub const FEATURE: &'static str = "game.block_material";

    /// The layout entry, optional under [`MaterialLayout::FEATURE`]. Call [`MaterialLayout::of`] after resolving
    /// it: a mod that needs block materials requires a layout.
    pub fn wanted() -> Vec<(EntryId, EntryNeed)> {
        vec![(EntryId::BLOCK_TYPE_MATERIAL_INLINE, EntryNeed::Optional { feature: Self::FEATURE })]
    }

    /// Whether `feature` is off by design on some builds. No feature is: the layout entry resolves on every
    /// build, so this is false for every feature.
    pub fn is_layout_feature(_feature: &str) -> bool {
        false
    }

    /// The layout the entry selects. Fails when the entry did not resolve or has a value that no mirror has.
    pub fn of(entries: &GameEntries) -> Result<Self, GameError> {
        match entries.get(EntryId::BLOCK_TYPE_MATERIAL_INLINE) {
            Some(0) => Ok(Self::Pointer),
            Some(1) => Ok(Self::Value),
            Some(value) => Err(Self::error(format!(
                "{} is {value:#x}, which no BlockType mirror has",
                EntryId::BLOCK_TYPE_MATERIAL_INLINE.as_str()
            ))),
            None => Err(Self::error(format!("{} did not resolve", EntryId::BLOCK_TYPE_MATERIAL_INLINE.as_str()))),
        }
    }

    fn error(detail: String) -> GameError {
        GameError::EntrySet { group: "block material", detail }
    }
}
