/// Why the ghost entity book refuses a call.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum GhostEntityRefusal {
    #[error("the ghost is of an older generation, or was freed or dropped")]
    Stale,
    #[error("the ghost belongs to another mod")]
    NotOwner,
    #[error("the mod holds the most ghosts it may")]
    TooMany,
}
