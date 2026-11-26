/// Why the item draw book refuses a call.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum ItemDrawRefusal {
    #[error("the draw is of an older generation, or was freed")]
    Stale,
    #[error("the draw belongs to another mod")]
    NotOwner,
    #[error("the mod holds the most draws it may")]
    TooMany,
}
