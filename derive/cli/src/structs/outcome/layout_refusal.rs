use amml_data::EntryId;

/// An entry that is not for this build: one of its `entry_is` checks names an entry that has another value here.
/// The entry belongs to the other layout of the game, so it is refused by design and is not missing.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("entry_is {guard} = {wanted:#x}; this build has {found:#x}")]
pub struct LayoutRefusal {
    /// The entry the check reads (for example `BlockType.mMaterialInline`).
    pub guard: EntryId,
    /// The value the check wants.
    pub wanted: u64,
    /// The value the guard has on this build.
    pub found: u64,
}
