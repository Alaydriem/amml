/// Why the group book refuses a call.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum GroupRefusal {
    #[error("the group is of an older generation, or was finished or dropped")]
    Stale,
    #[error("the group belongs to another mod")]
    NotOwner,
    #[error("the mod holds the most groups it may")]
    TooMany,
}
