/// Why the resource store refuses an id.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum ResourceError {
    #[error("the resource was freed, or its world or owner is gone")]
    Stale,
    #[error("the resource is of another kind")]
    WrongKind,
}
